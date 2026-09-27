import puppeteer, { type Browser, type Page } from 'puppeteer-core';
import { existsSync, readFileSync } from 'node:fs';
import { parseArgs } from 'node:util';

interface MediaItem {
  type: 'image' | 'video';
  url: string;
}

interface ScrapeResult {
  status: 'success' | 'error';
  platform?: string;
  title?: string;
  description?: string;
  media?: MediaItem[];
  error?: string;
}

function findChromiumPath(customPath?: string): string {
  if (customPath && existsSync(customPath)) {
    return customPath;
  }

  const candidates = [
    process.env.PUPPETEER_EXECUTABLE_PATH,
    process.env.CHROME_BIN,
    '/usr/bin/chromium',
    '/usr/bin/chromium-browser',
    '/usr/bin/google-chrome',
    '/usr/bin/google-chrome-stable',
    '/home/linuxbrew/.linuxbrew/bin/chromium',
  ];

  for (const candidate of candidates) {
    if (candidate && existsSync(candidate)) {
      return candidate;
    }
  }

  return '/usr/bin/chromium';
}

function parseCookies(cookiesPath: string): Array<{
  name: string;
  value: string;
  domain: string;
  path: string;
  secure?: boolean;
  httpOnly?: boolean;
  expires?: number;
}> {
  if (!existsSync(cookiesPath)) {
    return [];
  }

  const content = readFileSync(cookiesPath, 'utf-8').trim();
  if (!content) return [];

  // Try JSON format
  if (content.startsWith('[') || content.startsWith('{')) {
    try {
      const parsed = JSON.parse(content);
      const list = Array.isArray(parsed) ? parsed : [parsed];
      return list
        .filter((c: any) => c && c.name && c.value)
        .map((c: any) => ({
          name: String(c.name),
          value: String(c.value),
          domain: String(c.domain || '').replace(/^\./, ''),
          path: String(c.path || '/'),
          secure: Boolean(c.secure),
          httpOnly: Boolean(c.httpOnly),
          expires: typeof c.expirationDate === 'number' ? c.expirationDate : undefined,
        }));
    } catch {
      // Fall through to Netscape parsing
    }
  }

  // Netscape format
  const lines = content.split('\n');
  const cookies: Array<{
    name: string;
    value: string;
    domain: string;
    path: string;
    secure?: boolean;
    expires?: number;
  }> = [];

  for (const line of lines) {
    const trimmed = line.trim();
    if (!trimmed || trimmed.startsWith('#')) continue;

    const parts = trimmed.split('\t');
    if (parts.length >= 7) {
      let domain = parts[0].trim();
      if (domain.startsWith('#HttpOnly_')) {
        domain = domain.substring('#HttpOnly_'.length);
      }
      domain = domain.replace(/^\./, '');
      const path = parts[2].trim() || '/';
      const secure = parts[3].trim().toUpperCase() === 'TRUE';
      const expires = Number.parseInt(parts[4].trim(), 10);
      const name = parts[5].trim();
      const value = parts[6].trim();

      if (name) {
        cookies.push({
          name,
          value,
          domain,
          path,
          secure,
          expires: Number.isNaN(expires) || expires <= 0 ? undefined : expires,
        });
      }
    }
  }

  return cookies;
}

function detectPlatform(urlStr: string): string {
  try {
    const parsed = new URL(urlStr);
    const host = parsed.hostname.toLowerCase();
    if (host.includes('twitter.com') || host.includes('x.com')) return 'twitter';
    if (host.includes('instagram.com')) return 'instagram';
    if (host.includes('tiktok.com')) return 'tiktok';
    if (host.includes('threads.net')) return 'threads';
    if (host.includes('reddit.com') || host.includes('redd.it')) return 'reddit';
    if (host.includes('facebook.com') || host.includes('fb.watch')) return 'facebook';
    if (host.includes('pinterest.com') || host.includes('pin.it')) return 'pinterest';
    return 'generic';
  } catch {
    return 'generic';
  }
}

async function scrape(targetUrl: string, cookiesPath?: string, customChrome?: string, timeoutMs = 25000): Promise<ScrapeResult> {
  const platform = detectPlatform(targetUrl);
  const executablePath = findChromiumPath(customChrome);

  let browser: Browser | null = null;
  try {
    browser = await puppeteer.launch({
      executablePath,
      headless: true,
      args: [
        '--no-sandbox',
        '--disable-setuid-sandbox',
        '--disable-dev-shm-usage',
        '--disable-gpu',
        '--disable-accelerated-2d-canvas',
        '--no-first-run',
        '--no-zygote',
        '--single-process',
        '--disable-background-networking',
      ],
    });

    const page: Page = await browser.newPage();
    await page.setUserAgent(
      'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36',
    );
    await page.setViewport({ width: 1280, height: 800 });

    if (cookiesPath) {
      const cookies = parseCookies(cookiesPath);
      if (cookies.length > 0) {
        try {
          await page.setCookie(...cookies);
        } catch (cookieErr) {
          // Continue even if some cookies fail domain matching
        }
      }
    }

    const interceptedMedia: MediaItem[] = [];
    let apiTitle = '';
    let apiDescription = '';

    // Listen to network responses for social media JSON endpoints
    page.on('response', async (response) => {
      try {
        const respUrl = response.url();
        const contentType = response.headers()['content-type'] || '';

        // Twitter GraphQL API responses
        if (platform === 'twitter' && respUrl.includes('/graphql/') && contentType.includes('application/json')) {
          const json: any = await response.json();
          extractTwitterGraphData(json, interceptedMedia, (t, d) => {
            if (t && !apiTitle) apiTitle = t;
            if (d && !apiDescription) apiDescription = d;
          });
        }

        // Instagram GraphQL / query responses
        if (platform === 'instagram' && (respUrl.includes('/graphql/query') || respUrl.includes('/api/v1/media')) && contentType.includes('application/json')) {
          const json: any = await response.json();
          extractInstagramGraphData(json, interceptedMedia, (t, d) => {
            if (t && !apiTitle) apiTitle = t;
            if (d && !apiDescription) apiDescription = d;
          });
        }

        // Reddit JSON responses
        if (platform === 'reddit' && respUrl.includes('.json') && contentType.includes('application/json')) {
          const json: any = await response.json();
          extractRedditData(json, interceptedMedia, (t, d) => {
            if (t && !apiTitle) apiTitle = t;
            if (d && !apiDescription) apiDescription = d;
          });
        }
      } catch {
        // Ignore JSON parse errors on network responses
      }
    });

    // Navigate to page
    try {
      await page.goto(targetUrl, {
        waitUntil: 'networkidle2',
        timeout: timeoutMs,
      });
    } catch {
      // If networkidle2 timed out, still proceed to evaluate DOM content
    }

    // Wait a brief moment for dynamic client rendering
    await new Promise((r) => setTimeout(r, 1200));

    // Extract DOM information
    const domData = await page.evaluate((plat) => {
      const getMeta = (prop: string) =>
        document.querySelector(`meta[property="${prop}"]`)?.getAttribute('content') ||
        document.querySelector(`meta[name="${prop}"]`)?.getAttribute('content') ||
        '';

      const ogTitle = getMeta('og:title') || document.title || '';
      const ogDesc = getMeta('og:description') || getMeta('description') || '';
      const ogVideo = getMeta('og:video') || getMeta('og:video:url') || getMeta('og:video:secure_url') || '';
      const ogImage = getMeta('og:image') || getMeta('twitter:image') || '';

      const images: string[] = [];
      const videos: string[] = [];

      // Platform-specific DOM extraction
      if (plat === 'twitter') {
        const tweetTextElem = document.querySelector('article[data-testid="tweet"] div[data-testid="tweetText"]');
        const text = tweetTextElem?.textContent || '';

        // Media images in tweet
        const imgs = document.querySelectorAll('article[data-testid="tweet"] img[src*="pbs.twimg.com/media"]');
        imgs.forEach((img: any) => {
          let src = img.getAttribute('src') || '';
          if (src) {
            // Upgrade to orig or large resolution
            src = src.replace(/name=\w+/, 'name=orig');
            images.push(src);
          }
        });

        // Videos in tweet
        const vids = document.querySelectorAll('article[data-testid="tweet"] video');
        vids.forEach((v: any) => {
          const src = v.getAttribute('src') || v.querySelector('source')?.getAttribute('src');
          if (src && !src.startsWith('blob:')) {
            videos.push(src);
          }
        });

        return { title: text, description: '', images, videos, ogTitle, ogDesc, ogVideo, ogImage };
      }

      if (plat === 'instagram') {
        const vids = document.querySelectorAll('video');
        vids.forEach((v: any) => {
          const src = v.getAttribute('src') || v.querySelector('source')?.getAttribute('src');
          if (src && !src.startsWith('blob:')) videos.push(src);
        });

        const imgs = document.querySelectorAll('article img[srcset], main img[srcset]');
        imgs.forEach((img: any) => {
          const src = img.getAttribute('src');
          if (src && !src.includes('150x150') && !src.includes('profile')) images.push(src);
        });

        return { title: ogTitle, description: ogDesc, images, videos, ogTitle, ogDesc, ogVideo, ogImage };
      }

      if (plat === 'reddit') {
        const postElem = document.querySelector('shreddit-post');
        const title = postElem?.getAttribute('post-title') || document.querySelector('h1')?.textContent || ogTitle;
        const text = document.querySelector('div[slot="text-body"]')?.textContent || ogDesc;

        const img = document.querySelector('shreddit-post img[src*="redd.it"], shreddit-post img[src*="redditmedia"]');
        if (img) {
          const src = img.getAttribute('src');
          if (src) images.push(src);
        }

        const galleryImgs = document.querySelectorAll('gallery-carousel img');
        galleryImgs.forEach((gi: any) => {
          const src = gi.getAttribute('src');
          if (src) images.push(src);
        });

        const vid = document.querySelector('shreddit-player, video');
        if (vid) {
          const src = vid.getAttribute('src') || vid.querySelector('source')?.getAttribute('src');
          if (src && !src.startsWith('blob:')) videos.push(src);
        }

        return { title, description: text, images, videos, ogTitle, ogDesc, ogVideo, ogImage };
      }

      if (plat === 'tiktok') {
        const text = document.querySelector('[data-e2e="browse-video-desc"], [data-e2e="user-post-item-desc"]')?.textContent || ogTitle;
        const vids = document.querySelectorAll('video');
        vids.forEach((v: any) => {
          const src = v.getAttribute('src') || v.querySelector('source')?.getAttribute('src');
          if (src && !src.startsWith('blob:')) videos.push(src);
        });
        return { title: text, description: '', images, videos, ogTitle, ogDesc, ogVideo, ogImage };
      }

      if (plat === 'threads') {
        const text = document.querySelector('div[data-pressable-container="true"]')?.textContent || ogDesc || ogTitle;
        const imgs = document.querySelectorAll('img[src*="cdninstagram.com"], img[src*="fbcdn.net"]');
        imgs.forEach((img: any) => {
          const src = img.getAttribute('src');
          if (src && !src.includes('150x150') && !src.includes('profile')) images.push(src);
        });
        const vids = document.querySelectorAll('video');
        vids.forEach((v: any) => {
          const src = v.getAttribute('src') || v.querySelector('source')?.getAttribute('src');
          if (src && !src.startsWith('blob:')) videos.push(src);
        });
        return { title: text, description: '', images, videos, ogTitle, ogDesc, ogVideo, ogImage };
      }

      if (plat === 'pinterest') {
        const text = document.querySelector('h1')?.textContent || ogTitle;
        const desc = document.querySelector('[data-test-id="pin-description"]')?.textContent || ogDesc;
        const imgs = document.querySelectorAll('img[src*="pinimg.com/originals/"], img[src*="pinimg.com/736x/"]');
        imgs.forEach((img: any) => {
          const src = img.getAttribute('src');
          if (src) images.push(src.replace(/\/736x\//, '/originals/'));
        });
        return { title: text, description: desc, images, videos, ogTitle, ogDesc, ogVideo, ogImage };
      }

      // Generic DOM extraction
      const allVideos = document.querySelectorAll('video');
      allVideos.forEach((v: any) => {
        const src = v.getAttribute('src') || v.querySelector('source')?.getAttribute('src');
        if (src && !src.startsWith('blob:')) videos.push(src);
      });

      const allImgs = document.querySelectorAll('img');
      allImgs.forEach((img: any) => {
        const src = img.currentSrc || img.getAttribute('src');
        const w = img.naturalWidth || img.clientWidth || 0;
        const h = img.naturalHeight || img.clientHeight || 0;
        if (src && (w >= 300 || h >= 300) && !src.startsWith('data:')) {
          images.push(src);
        }
      });

      return { title: ogTitle, description: ogDesc, images, videos, ogTitle, ogDesc, ogVideo, ogImage };
    }, platform);

    // Combine intercepted and DOM media
    const finalMedia: MediaItem[] = [];
    const seenUrls = new Set<string>();

    const addMedia = (type: 'image' | 'video', url: string) => {
      if (!url || typeof url !== 'string') return;
      const clean = url.trim();
      if (!clean.startsWith('http://') && !clean.startsWith('https://')) return;
      if (seenUrls.has(clean)) return;
      seenUrls.add(clean);
      finalMedia.push({ type, url: clean });
    };

    // Prioritize videos over images if video was found
    for (const v of interceptedMedia.filter((m) => m.type === 'video')) addMedia('video', v.url);
    for (const v of domData.videos) addMedia('video', v);
    if (domData.ogVideo) addMedia('video', domData.ogVideo);

    for (const img of interceptedMedia.filter((m) => m.type === 'image')) addMedia('image', img.url);
    for (const img of domData.images) addMedia('image', img);
    if (finalMedia.length === 0 && domData.ogImage) addMedia('image', domData.ogImage);

    let title = (apiTitle || domData.title || '').trim();
    let description = (apiDescription || domData.description || '').trim();

    if (!title && domData.ogDesc) {
      title = domData.ogDesc.trim();
    } else if (!title && domData.ogTitle) {
      title = domData.ogTitle.trim();
    }

    // If title is just a platform user container like "... on X" or "... on Instagram", prefer description
    if (/(?:on\s+X|on\s+Twitter|on\s+Instagram|\/\s*X)$/i.test(title) && description) {
      title = description;
      description = '';
    } else if (/(?:on\s+X|on\s+Twitter|on\s+Instagram|\/\s*X)$/i.test(title) && domData.ogDesc) {
      title = domData.ogDesc.trim();
    }

    return {
      status: 'success',
      platform,
      title,
      description,
      media: finalMedia,
    };
  } catch (err: any) {
    return {
      status: 'error',
      platform,
      error: err?.message || String(err),
    };
  } finally {
    if (browser) {
      await browser.close().catch(() => {});
    }
  }
}

function extractTwitterGraphData(
  json: any,
  mediaList: MediaItem[],
  setText: (title: string, desc: string) => void,
) {
  try {
    const parseEntities = (legacy: any) => {
      if (!legacy) return;
      if (legacy.full_text) {
        setText(legacy.full_text, '');
      }
      const media = legacy.extended_entities?.media || legacy.entities?.media || [];
      for (const m of media) {
        if (m.type === 'photo' && m.media_url_https) {
          mediaList.push({ type: 'image', url: `${m.media_url_https}?name=orig` });
        } else if ((m.type === 'video' || m.type === 'animated_gif') && m.video_info?.variants) {
          const mp4s = m.video_info.variants
            .filter((v: any) => v.content_type === 'video/mp4' && v.url)
            .sort((a: any, b: any) => (b.bitrate || 0) - (a.bitrate || 0));
          if (mp4s.length > 0) {
            mediaList.push({ type: 'video', url: mp4s[0].url });
          }
        }
      }
    };

    const traverse = (obj: any) => {
      if (!obj || typeof obj !== 'object') return;
      if (obj.legacy) parseEntities(obj.legacy);
      if (obj.tweet?.legacy) parseEntities(obj.tweet.legacy);
      for (const key of Object.keys(obj)) {
        if (typeof obj[key] === 'object') traverse(obj[key]);
      }
    };

    traverse(json);
  } catch {
    // Ignore extraction errors
  }
}

function extractInstagramGraphData(
  json: any,
  mediaList: MediaItem[],
  setText: (title: string, desc: string) => void,
) {
  try {
    const traverse = (obj: any) => {
      if (!obj || typeof obj !== 'object') return;
      if (obj.video_url && typeof obj.video_url === 'string') {
        mediaList.push({ type: 'video', url: obj.video_url });
      }
      if (obj.video_versions && Array.isArray(obj.video_versions) && obj.video_versions.length > 0) {
        mediaList.push({ type: 'video', url: obj.video_versions[0].url });
      }
      if (obj.image_versions2?.candidates && Array.isArray(obj.image_versions2.candidates) && obj.image_versions2.candidates.length > 0) {
        mediaList.push({ type: 'image', url: obj.image_versions2.candidates[0].url });
      }
      if (obj.display_url && typeof obj.display_url === 'string') {
        mediaList.push({ type: 'image', url: obj.display_url });
      }
      if (obj.caption?.text && typeof obj.caption.text === 'string') {
        setText(obj.caption.text, '');
      }
      for (const k of Object.keys(obj)) {
        if (typeof obj[k] === 'object') traverse(obj[k]);
      }
    };
    traverse(json);
  } catch {
    // Ignore
  }
}

function extractRedditData(
  json: any,
  mediaList: MediaItem[],
  setText: (title: string, desc: string) => void,
) {
  try {
    const list = Array.isArray(json) ? json : [json];
    for (const item of list) {
      const children = item?.data?.children || [];
      for (const child of children) {
        const post = child.data;
        if (!post) continue;
        if (post.title) setText(post.title, post.selftext || '');

        if (post.media?.reddit_video?.fallback_url) {
          mediaList.push({ type: 'video', url: post.media.reddit_video.fallback_url });
        }
        if (post.url_overridden_by_dest) {
          const dest = post.url_overridden_by_dest;
          if (/\.(jpg|jpeg|png|webp|gif)$/i.test(dest)) {
            mediaList.push({ type: 'image', url: dest });
          }
        }
      }
    }
  } catch {
    // Ignore
  }
}

async function main() {
  const { values } = parseArgs({
    args: Bun.argv.slice(2),
    options: {
      url: { type: 'string' },
      cookies: { type: 'string' },
      chrome: { type: 'string' },
      timeout: { type: 'string' },
    },
    strict: false,
    allowPositionals: true,
  });

  const url = values.url || Bun.argv[2];
  if (!url) {
    console.log(JSON.stringify({ status: 'error', error: 'Missing --url argument' }));
    process.exit(1);
  }

  const cookies = values.cookies as string | undefined;
  const chrome = values.chrome as string | undefined;
  const timeoutMs = values.timeout ? Number.parseInt(values.timeout as string, 10) : 25000;

  const result = await scrape(url, cookies, chrome, timeoutMs);
  console.log(JSON.stringify(result));
}

main().catch((err) => {
  console.log(JSON.stringify({ status: 'error', error: String(err) }));
  process.exit(1);
});
