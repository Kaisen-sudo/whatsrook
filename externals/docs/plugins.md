# Plugin Reference Catalog

WhatsRook external plugins provide 29 specialized commands covering media transformations, financial tickers, system monitoring, utility calculations, and AI intelligence.

---

## Quick Command Matrix

| Command | Category | Permissions | Description |
| :--- | :--- | :--- | :--- |
| [`.weather`](#weather) | Utility | Everyone | Real-time weather forecasts by city or coordinates |
| [`.calc`](#calculator) | Utility | Everyone | Scientific calculator & mathematical expression parser |
| [`.shorturl`](#url-shortener) | Utility | Everyone | URL shortener via TinyURL and is.gd |
| [`.ss`](#webpage-screenshot) | Utility | Everyone | High-resolution webpage screenshot renderer |
| [`.qrcode`](#qr-code-generator) | Utility | Everyone | Generate high-res QR code PNG images |
| [`.captcha`](#dial-video-captcha) | Utility | Sudo / Owner | Animated dial verification code video captchas |
| [`.cpu`](#cpu-diagnostics) | System | Sudo / Owner | Host CPU architecture, core usage, and load averages |
| [`.memory`](#memory-diagnostics) | System | Sudo / Owner | Host RAM and swap utilization diagnostics |
| [`.sticker`](#sticker) | Media | Everyone | Converts media to letterboxed 512×512 WebP sticker |
| [`.circle`](#circle-sticker) | Media | Everyone | Converts media to circular-masked WebP sticker |
| [`.crop`](#crop-sticker) | Media | Everyone | Converts media to square-cropped WebP sticker |
| [`.take`](#take-metadata-re-packer) | Media | Everyone | Re-packs sticker metadata (author and pack name) |
| [`.media`](#media-transcoder) | Media | Everyone | Video/audio format converter, trimmer, and extractor |
| [`.mp4url`](#mp4-video-streamer) | Media | Everyone | Streams video directly from web MP4 URLs |
| [`.tts`](#text-to-speech) | Media | Everyone | Google Text-to-Speech audio voice notes |
| [`.fancy`](#fancy-text) | Styling | Everyone | Converts plain text into 20+ decorative Unicode fonts |
| [`.font`](#numbered-font) | Styling | Everyone | Formats text into a specific numbered font style |
| [`.fonts`](#fonts-directory) | Styling | Everyone | Visual directory and sample catalog of all font styles |
| [`.fact`](#random-facts) | Fun | Everyone | Random interesting facts with offline fallbacks |
| [`.quotes`](#inspirational-quotes) | Fun | Everyone | Inspirational quotes with author attributions |
| [`.joke`](#jokes) | Fun | Everyone | Witty jokes and punchlines |
| [`.rizz`](#rizz-pickup-lines) | Fun | Everyone | Charismatic pickup lines and modern compliments |
| [`.btc`](#bitcoin-live-ticker) | Finance | Everyone | Live updating Bitcoin price, 24h change & halving metrics |
| [`.markets`](#forex-commodities) | Finance | Everyone | Forex, commodity, crypto, and market index rates |
| [`.news`](#breaking-news) | News | Everyone | AP News top headlines filtered by country code |
| [`.wabeta`](#whatsapp-beta-leaks) | News | Everyone | WhatsApp beta feature leaks from WABetaInfo |
| [`.why`](#why-ai-search) | AI | Everyone | AI-powered deep exploration and knowledge search |
| [`.git`](#github-explorer) | Developer | Everyone | GitHub repository explorer, commits, releases & ZIP |

---

## 🛠️ System & Diagnostics

### CPU Diagnostics
Inspect host CPU model, physical and logical cores, architecture, and current load.

- **Trigger**: `.cpu`
- **Permissions**: Owner / Sudoers
- **Example**:
    ```text
    .cpu
    ```
- **Output Preview**:
    ```text
    🖥️ *CPU Information*

    • *Model:* AMD Ryzen 9 7950X 16-Core Processor
    • *Architecture:* x86_64
    • *Cores:* 16 Physical / 32 Logical
    • *Usage:* 14.2%
    • *Load Average:* 1.25, 1.40, 1.18
    ```

### Memory Diagnostics
Examine host RAM utilization, available memory buffers, and swap consumption.

- **Trigger**: `.memory`
- **Permissions**: Owner / Sudoers
- **Example**:
    ```text
    .memory
    ```
- **Output Preview**:
    ```text
    💾 *System Memory*

    • *RAM Used:* 8.42 GB / 31.25 GB (26.9%)
    • *RAM Available:* 22.83 GB
    • *Swap Used:* 0.50 GB / 8.00 GB (6.2%)
    ```

---

## 🌐 General Utilities

### Weather
Fetch real-time weather forecasts, current temperature, humidity, and wind conditions from `wttr.in`.

- **Trigger**: `.weather <city | coordinates>`
- **Example**:
    ```text
    .weather Tokyo
    .weather Paris, France
    .weather 40.7128,-74.0060
    ```
- **Output Preview**:
    ```text
    Tokyo: ⛅️  +19°C ↗ 14km/h 10km 0.0mm 62% 1016hPa
    ```

### Calculator
High-precision mathematical and scientific expression evaluator. Supports arithmetic operators, scientific constants (`pi`, `e`), trigonometric functions (`sin`, `cos`, `tan`), logarithms (`ln`, `log10`), and powers.

- **Trigger**: `.calc <expression>`
- **Example**:
    ```text
    .calc sqrt(256) + 2^5
    .calc sin(pi / 4) * cos(pi / 4)
    .calc log10(10000) * e^2
    ```
- **Output Preview**:
    ```text
    🔢 *Calculation*

    *Expression:* sqrt(256) + 2^5
    *Result:* 48
    ```

### URL Shortener
Shorten long or unwieldy URLs into compact links via TinyURL and is.gd services.

- **Trigger**: `.shorturl <url>`
- **Example**:
    ```text
    .shorturl https://github.com/Thruqe/whatsrook-externals/blob/master/Cargo.toml
    ```
- **Output Preview**:
    ```text
    🔗 *Shortened URL:* https://tinyurl.com/wr-cargo
    ```

### Webpage Screenshot
Captures a full-page, high-resolution desktop screenshot of any valid HTTP/HTTPS webpage.

- **Trigger**: `.ss <url>`
- **Example**:
    ```text
    .ss https://news.ycombinator.com
    ```
- **Output**: Returns an image message with the rendered screenshot as an attachment.

### QR Code Generator
Generates a crisp, scannable QR code PNG image from arbitrary input text, phone numbers, or URLs.

- **Trigger**: `.qrcode <text | url>`
- **Example**:
    ```text
    .qrcode https://github.com/Thruqe/whatsrook
    ```
- **Output**: Returns a QR code PNG image.

### Dial Video Captcha
Generates an animated dial verification video captcha containing randomized numeric codes, ideal for verifying new group joiners.

- **Trigger**: `.captcha [code]`
- **Permissions**: Owner / Sudoers
- **Requirements**: `ffmpeg` installed on the host.

---

## 🎨 Media & Stickers

!!! info "Quoted Reply Triggering"
    For sticker and media commands, send the command as a **reply (quote)** to an existing image, video, or sticker message in the chat.

### Sticker
Converts replied-to images, GIFs, or short videos (up to 10 seconds) into a letterboxed 512×512 WebP sticker. Includes genuine WhatsApp EXIF sticker pack metadata.

- **Trigger**: `.sticker [pack_name] | [author]`
- **Examples**:
    ```text
    .sticker
    .sticker Custom Pack | Thruqe
    ```

### Circle Sticker
Applies a circular alpha mask to the replied-to media and outputs a circular sticker with transparent corners.

- **Trigger**: `.circle [pack_name] | [author]`
- **Example**:
    ```text
    .circle
    .circle Avatar Pack | WhatsRook
    ```

### Crop Sticker
Crops the replied-to media to a 1:1 square centered frame without letterboxing bars.

- **Trigger**: `.crop [pack_name] | [author]`

### Take (Metadata Re-Packer)
Clones an existing sticker from another pack, stripping its original EXIF data and stamping your custom author and pack name.

- **Trigger**: `.take [pack_name] | [author]`
- **Example**:
    ```text
    .take Stolen Memes | Alex
    ```

### Media Transcoder
Multi-purpose video and audio processor. Trims media, extracts audio tracks into voice notes (PTT), or converts formats.

- **Trigger**: `.media <audio | mp3 | trim <start> <end>>`
- **Examples**:
    ```text
    .media audio         # Extract audio stream as MP3
    .media trim 00:05 00:15 # Cut segment between 5s and 15s
    ```

### MP4 Video Streamer
Downloads and streams media directly from web `.mp4` URLs directly into the WhatsApp conversation.

- **Trigger**: `.mp4url <url>`

### Text-To-Speech
Converts text into natural-sounding spoken audio notes in dozens of languages via Google TTS.

- **Trigger**: `.tts <lang_code> <text>`
- **Examples**:
    ```text
    .tts en Welcome to WhatsRook external plugins!
    .tts es Hola, ¿cómo estás hoy?
    .tts ja こんにちは世界
    ```

---

## 🎭 Styling & Decorative Fonts

### Fancy Text
Transforms regular Latin text into over 20 stylistic Unicode font variations simultaneously (Gothic, Script, Double-Struck, Circles, Bubbles, Monospace, etc.).

- **Trigger**: `.fancy <text>`
- **Example**:
    ```text
    .fancy WhatsRook
    ```
- **Output Preview**:
    ```text
    1. 𝖂𝖍𝖆𝖙𝖘𝕽𝖔𝖔𝖐
    2. 𝓦𝓱𝓪𝓽𝓼𝓡𝓸𝓸𝓴
    3. 𝕎𝕙𝕒𝕥𝕤ℝ𝕠𝕠𝕜
    4. ⓌⓗⓐⓣⓢⓇⓞⓞⓚ
    5. 🅆🄷🄰🅃🅂🅁🄾🄾🄺
    ```

### Numbered Font
Converts text directly into one chosen font index from the `.fonts` directory.

- **Trigger**: `.font <index> <text>`
- **Example**:
    ```text
    .font 3 WhatsRook
    ```

### Fonts Directory
Lists every supported Unicode decorative font style with sample previews.

- **Trigger**: `.fonts`

---

## 💡 Fun & Knowledge

### Random Facts
Retrieves verified, fascinating facts from educational databases. Automatically provides curated offline fallbacks if the remote service is unavailable.

- **Trigger**: `.fact`
- **Output Preview**:
    ```text
    🧠 *Did You Know?*
    Honey never spoils. Archaeologists have found pots of honey in ancient Egyptian tombs that are over 3,000 years old and still edible.
    ```

### Inspirational Quotes
Returns notable quotes accompanied by author attributions and philosophical themes.

- **Trigger**: `.quotes`

### Jokes
Clean, lighthearted jokes, programming humor, and witty one-liners.

- **Trigger**: `.joke`

### Rizz (Pickup Lines)
Humorous modern pickup lines, witty openers, and charismatic dialogue.

- **Trigger**: `.rizz`

---

## 📈 Finance & Crypto

### Bitcoin Live Ticker
A **streaming live session** plugin! Emits real-time Bitcoin spot prices from Binance and WatcherGuru, updating the sent message in-place every 1.5 seconds. Displays 24h price changes, current block height, and block countdown to halving.

- **Trigger**: `.btc`
- **Live Duration**: Up to 5 minutes continuous auto-refresh.
- **Output Preview**:
    ```text
    ₿ *Bitcoin (BTC)*

    *Price:* $98,420.50 (📈 +3.45%)
    *Current Block:* 889,120
    *Halving Target:* 1,050,000
    *Blocks Remaining:* 160,880

    _Auto-refreshing live (1.5s)_
    ```

### Forex & Commodities
Real-time financial exchange rates across currencies (`EUR/USD`, `GBP/JPY`), commodities (`Gold`, `Silver`, `Oil`), and major equity indices (`SPX500`, `NAS100`).

- **Trigger**: `.markets <pair>`
- **Example**:
    ```text
    .markets EUR/USD
    .markets Gold
    ```

---

## 📰 News & Updates

### Breaking News
Retrieves curated top breaking news headlines from the Associated Press (AP News), organized with publication timestamps and direct web source links.

- **Trigger**: `.news [country_code]`
- **Examples**:
    ```text
    .news
    .news us
    .news gb
    ```

### WhatsApp Beta Leaks
Scrapes and parses the latest feature releases, experimental interface changes, and security advisories published by WABetaInfo.

- **Trigger**: `.wabeta`

---

## 🧠 AI & Developer Tools

### Why AI Search
Queries the Why.com AI search engine to synthesize answers to deep exploratory and analytical questions.

- **Trigger**: `.why <question>`
- **Example**:
    ```text
    .why Why does Rust enforce strict memory borrowing at compile time?
    ```

### GitHub Explorer
Fetches details on any public GitHub repository: star count, fork metrics, open issues, primary language, recent commit logs, latest release tags, and direct ZIP download links.

- **Trigger**: `.git <owner/repo>`
- **Example**:
    ```text
    .git Thruqe/whatsrook
    .git rust-lang/rust
    ```
