package dl

import (
	"context"
	"encoding/json"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"time"

	"whatsrook"
	"whatsrook/cmd/dispatch"
	"whatsrook/util/httpx"
	"whatsrook/util/logger"
	"whatsrook/util/media"
)

// ScraperMediaItem represents an image or video found by the headless scraper.
type ScraperMediaItem struct {
	Type string `json:"type"` // "image" or "video"
	URL  string `json:"url"`
}

// ScraperResult represents the structured JSON output from the Bun scraper.
type ScraperResult struct {
	Status      string             `json:"status"`
	Platform    string             `json:"platform"`
	Title       string             `json:"title"`
	Description string             `json:"description"`
	Media       []ScraperMediaItem `json:"media"`
	Error       string             `json:"error"`
}

func findBunPath() (string, error) {
	if p, err := exec.LookPath("bun"); err == nil {
		return p, nil
	}
	candidates := []string{
		"/home/linuxbrew/.linuxbrew/bin/bun",
		"/usr/local/bin/bun",
		"/usr/bin/bun",
	}
	for _, c := range candidates {
		if _, err := os.Stat(c); err == nil {
			return c, nil
		}
	}
	return "", fmt.Errorf("bun runtime was not found on system PATH")
}

func findScraperScript() (string, error) {
	// 1. Check relative to current working directory
	relPaths := []string{
		filepath.Join("cmd", "dl", "scraper", "scrape.ts"),
		filepath.Join(".", "cmd", "dl", "scraper", "scrape.ts"),
	}
	for _, p := range relPaths {
		if _, err := os.Stat(p); err == nil {
			return filepath.Abs(p)
		}
	}

	// 2. Check relative to executable directory
	if execPath, err := os.Executable(); err == nil {
		execDir := filepath.Dir(execPath)
		p := filepath.Join(execDir, "cmd", "dl", "scraper", "scrape.ts")
		if _, err := os.Stat(p); err == nil {
			return p, nil
		}
		// Also check parent dir (e.g. if running from ./bin)
		pParent := filepath.Join(filepath.Dir(execDir), "cmd", "dl", "scraper", "scrape.ts")
		if _, err := os.Stat(pParent); err == nil {
			return pParent, nil
		}
	}

	return "", fmt.Errorf("scraper script scrape.ts was not found")
}

// runSocialScraper executes the Bun + Puppeteer scraper to extract media from social media links.
func runSocialScraper(ctx *dispatch.Context, rawURL string) (*ScraperResult, error) {
	bunPath, errBun := findBunPath()
	if errBun != nil {
		return nil, errBun
	}

	scriptPath, errScript := findScraperScript()
	if errScript != nil {
		return nil, errScript
	}

	cookiesPath, cleanup := getCookiesFilePath(ctx, rawURL)
	defer cleanup()

	args := []string{"run", scriptPath, "--url", rawURL}
	if cookiesPath != "" {
		args = append(args, "--cookies", cookiesPath)
	}

	timeoutCtx, cancel := context.WithTimeout(ctx.GetSendContext(), 45*time.Second)
	defer cancel()

	cmd := exec.CommandContext(timeoutCtx, bunPath, args...)
	cmd.Dir = filepath.Dir(scriptPath)

	out, err := cmd.Output()
	if err != nil {
		return nil, fmt.Errorf("social scraper execution failed: %w", err)
	}

	var result ScraperResult
	if err := json.Unmarshal(out, &result); err != nil {
		return nil, fmt.Errorf("failed parsing scraper response: %w (%s)", err, strings.TrimSpace(string(out)))
	}

	if result.Status == "error" {
		return nil, fmt.Errorf("scraper error: %s", result.Error)
	}

	return &result, nil
}

// deliverScrapedMedia handles downloading and sending media items extracted by the social scraper.
func deliverScrapedMedia(ctx *dispatch.Context, result *ScraperResult) error {
	if len(result.Media) == 0 {
		return fmt.Errorf("no media items found")
	}

	caption := cleanTopicAndContext(result.Title, result.Description)

	// Single image delivery
	if len(result.Media) == 1 && result.Media[0].Type == "image" {
		dlCtx, cancel := context.WithTimeout(ctx.GetSendContext(), 30*time.Second)
		imgBytes, err := httpx.FetchBytes(dlCtx, result.Media[0].URL)
		cancel()
		if err != nil || len(imgBytes) == 0 {
			return fmt.Errorf("failed downloading image: %w", err)
		}

		jpegData, errConv := media.EnsureJPEG(ctx.GetSendContext(), imgBytes)
		if errConv == nil && len(jpegData) > 0 {
			imgBytes = jpegData
		}
		return ctx.ReplyWithImage(imgBytes, "image/jpeg", caption)
	}

	// Single video delivery
	if len(result.Media) == 1 && result.Media[0].Type == "video" {
		vidBytes, err := fetchAndTranscodeVideo(ctx, result.Media[0].URL)
		if err != nil {
			return err
		}
		return ctx.ReplyWithVideo(vidBytes, "video/mp4", caption)
	}

	// Multi-media delivery via WhatsApp album
	var albumItems []whatsrook.AlbumMediaItem
	for _, m := range result.Media {
		if m.Type == "video" {
			vidBytes, err := fetchAndTranscodeVideo(ctx, m.URL)
			if err != nil || len(vidBytes) == 0 {
				logger.Warn("deliverScrapedMedia: skipped video item", "url", m.URL, "err", err)
				continue
			}
			albumItems = append(albumItems, whatsrook.AlbumMediaItem{
				Data:     vidBytes,
				Mimetype: "video/mp4",
				IsVideo:  true,
			})
		} else {
			dlCtx, cancel := context.WithTimeout(ctx.GetSendContext(), 30*time.Second)
			imgBytes, err := httpx.FetchBytes(dlCtx, m.URL)
			cancel()
			if err != nil || len(imgBytes) == 0 {
				logger.Warn("deliverScrapedMedia: skipped image item", "url", m.URL, "err", err)
				continue
			}
			jpegData, errConv := media.EnsureJPEG(ctx.GetSendContext(), imgBytes)
			if errConv == nil && len(jpegData) > 0 {
				imgBytes = jpegData
			}
			albumItems = append(albumItems, whatsrook.AlbumMediaItem{
				Data:     imgBytes,
				Mimetype: "image/jpeg",
				IsVideo:  false,
			})
		}
	}

	if len(albumItems) == 0 {
		return fmt.Errorf("failed fetching any media items for delivery")
	}

	if len(albumItems) == 1 {
		it := albumItems[0]
		if it.IsVideo {
			return ctx.ReplyWithVideo(it.Data, it.Mimetype, caption)
		}
		return ctx.ReplyWithImage(it.Data, it.Mimetype, caption)
	}

	albumItems[0].Caption = caption
	return ctx.ReplyWithAlbum(albumItems)
}

// fetchAndTranscodeVideo downloads a video URL and ensures MP4/H.264 compatibility for WhatsApp.
func fetchAndTranscodeVideo(ctx *dispatch.Context, vidURL string) ([]byte, error) {
	nowNano := time.Now().UnixNano()
	rawPath := filepath.Join(os.TempDir(), fmt.Sprintf("scraped_vid_raw_%d.mp4", nowNano))
	mp4Out := filepath.Join(os.TempDir(), fmt.Sprintf("scraped_vid_out_%d.mp4", nowNano))

	defer func() {
		_ = os.Remove(rawPath)
		_ = os.Remove(mp4Out)
	}()

	dlCtx, cancel := context.WithTimeout(ctx.GetSendContext(), 2*time.Minute)
	data, err := httpx.FetchBytes(dlCtx, vidURL)
	cancel()
	if err != nil || len(data) == 0 {
		return nil, fmt.Errorf("failed downloading video: %w", err)
	}

	if err := os.WriteFile(rawPath, data, 0600); err != nil {
		return nil, fmt.Errorf("failed saving raw video: %w", err)
	}

	if err := EnsureWhatsAppVideo(ctx.GetSendContext(), rawPath, mp4Out); err != nil {
		logger.Warn("EnsureWhatsAppVideo failed, using original video stream", "err", err)
		return data, nil
	}

	outBytes, errRead := os.ReadFile(mp4Out)
	if errRead != nil || len(outBytes) == 0 {
		return data, nil
	}

	return outBytes, nil
}
