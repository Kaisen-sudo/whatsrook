# Plugin Reference Catalog

This reference covers all 29 official external plugins included in the workspace.

---

| Command | Plugin | Category | Description |
| :--- | :--- | :--- | :--- |
| `.weather` | `weather` | Utility | Real-time weather forecast by city name or location coordinates |
| `.urban` | `urban` | Reference | Urban Dictionary slang terms, meanings, and examples |
| `.shorturl` | `shorturl` | Utility | URL shortener using TinyURL and is.gd |
| `.calc` | `calc` | Math | High-precision mathematical and scientific expression evaluator |
| `.fact` | `fact` | Fun | Random interesting facts with offline fallbacks |
| `.quotes` | `quotes` | Fun | Inspirational quotes and attributed authors |
| `.joke` | `joke` | Fun | Clean, witty jokes and punchlines |
| `.rizz` | `rizz` | Fun | Modern pickup lines and charismatic phrases |
| `.btc` | `btc` | Finance | Real-time Bitcoin spot price, change, and halving block metrics |
| `.markets` | `markets` | Finance | Forex Factory market rates (Currencies, Commodities, Indices, Crypto) |
| `.news` | `news` | Media | Latest breaking news headlines from AP News by country code |
| `.wabeta` | `wabeta` | WhatsApp | Latest WhatsApp feature leaks and updates from WABetaInfo |
| `.why` | `why` | AI / Search | AI-powered deep exploration and knowledge search from Why.com |
| `.ss` | `ss` | Utility | High-resolution webpage full-page screenshots |
| `.tts` | `tts` | Media | Text-to-speech audio converter via Google TTS |
| `.qrcode` | `qrcode` | Utility | Generates high-res QR code PNG images from any text or URL |
| `.fancy` | `fancy` | Styling | Converts plain text into 20+ decorative Unicode fonts |
| `.font` | `font` | Styling | Formats input text into a specific numbered Unicode font |
| `.fonts` | `fonts` | Styling | Directory and visual samples of all supported font styles |
| `.git` | `git` | Developer | GitHub repository explorer, commits, releases, and ZIP archives |
| `.mp4url` | `mp4url` | Media | Downloads and streams video directly from web MP4 URLs |
| `.cpu` | `cpu` | System | Host CPU model, core architecture, active threads, and load |
| `.memory` | `memory` | System | Host RAM usage, available memory, and system resource metrics |
| `.captcha` | `captcha` | Utility | Generates animated dial verification code video captchas |
| `.sticker` | `sticker` | Media | Converts images/videos into letterboxed 512×512 WebP stickers |
| `.circle` | `circle` | Media | Converts images/videos into circular-masked WebP stickers |
| `.crop` | `crop` | Media | Converts images/videos into square-cropped WebP stickers |
| `.take` | `take` | Media | Re-packs sticker metadata — author name and pack name |
| `.media` | `media` | Media | Video/audio format converter, audio extractor, and trimmer |

---

## Detailed Usage Examples

### Weather (`.weather`)
```text
.weather Tokyo
.weather London, UK
.weather 37.7749,-122.4194
```

### Calculator (`.calc`)
Supports standard arithmetic, trigonometric functions, powers, roots, and constants:
```text
.calc sqrt(144) + 2^8
.calc sin(pi / 4) * cos(pi / 4)
.calc log10(1000) * ln(e^2)
```

### Market Rates (`.markets`)
```text
.markets EUR/USD
.markets Gold/USD
.markets SPX500
.markets BTC/USD
```

### Text-To-Speech (`.tts`)
Generates an audio note spoken in the specified language:
```text
.tts en Hello, welcome to WhatsRook!
.tts es Hola, ¿cómo estás hoy?
.tts fr Bonjour tout le monde!
```

### Webpage Screenshot (`.ss`)
Renders and returns a screenshot of the target URL:
```text
.ss https://news.ycombinator.com
.ss https://github.com
```

### GitHub Explorer (`.git`)
Inspect repositories or download releases:
```text
.git thruqe/whatsrook
.git torvalds/linux
```

### Sticker Plugins (`.sticker`, `.circle`, `.crop`, `.take`)
Reply to any image or video with the desired command:
```text
.sticker                    → letterboxed 512×512 sticker
.sticker Author | Pack      → sticker with custom metadata
.circle                     → circular-masked sticker
.crop                       → square-cropped sticker
.take                       → re-pack a quoted sticker's metadata
.take Author | Pack         → re-pack with specific author and pack name
```
