# WhatsRook External Plugins 🔌

Official suite of standalone, high-performance external plugins for **[WhatsRook](https://github.com/Thruqe/whatsrook)** written in Rust.

External plugins run as isolated child processes, communicating with WhatsRook via JSON over `stdin` and returning replies over `stdout`. They do not require recompilation or modifications to the WhatsRook codebase and can be installed or uninstalled dynamically at runtime via WhatsApp commands.

---

## ⚡ Instant 1-Click WhatsApp Installation

With WhatsRook's platform-aware installer, simply run:

```text
.install <command>
```

WhatsRook automatically detects your host platform and architecture (Linux, macOS, Windows; AMD64, ARM64) and completes the download URL.

To install all official plugins at once:
```text
.install all
```

---

## 📦 Available Plugins & Installation Links

Click the copy button on any block below to install directly from WhatsApp:

### 1. Weather (`weather`)
Real-time weather forecast for any city or town.
```text
.install weather https://github.com/Thruqe/whatsrook-externals/releases/latest/download/weather
```
*Short command:*
```text
.install weather
```

### 2. Urban Dictionary (`urban`)
Urban Dictionary slang and definition lookup.
```text
.install urban https://github.com/Thruqe/whatsrook-externals/releases/latest/download/urban
```
*Short command:*
```text
.install urban
```

### 3. URL Shortener (`shorturl`)
Shortens long URLs using TinyURL and is.gd.
```text
.install shorturl https://github.com/Thruqe/whatsrook-externals/releases/latest/download/shorturl
```
*Short command:*
```text
.install shorturl
```

### 4. Calculator & Math (`calc`)
Mathematical expression evaluator (`+`, `-`, `*`, `/`, `%`, `^`, `sqrt`, `sin`, `cos`, `tan`, `log`, `ln`, `pi`, `e`, etc.).
```text
.install calc https://github.com/Thruqe/whatsrook-externals/releases/latest/download/calc
```
*Short command:*
```text
.install calc
```

### 5. Random Facts (`fact`)
Interesting random facts from public APIs with offline fallbacks.
```text
.install fact https://github.com/Thruqe/whatsrook-externals/releases/latest/download/fact
```
*Short command:*
```text
.install fact
```

### 6. Inspirational Quotes (`quotes`)
Inspirational quotes and authors.
```text
.install quotes https://github.com/Thruqe/whatsrook-externals/releases/latest/download/quotes
```
*Short command:*
```text
.install quotes
```

### 7. Jokes (`joke`)
Clean jokes and funny punchlines.
```text
.install joke https://github.com/Thruqe/whatsrook-externals/releases/latest/download/joke
```
*Short command:*
```text
.install joke
```

### 8. Rizz & Pickup Lines (`rizz`)
Smooth pickup lines & rizz.
```text
.install rizz https://github.com/Thruqe/whatsrook-externals/releases/latest/download/rizz
```
*Short command:*
```text
.install rizz
```

### 9. Bitcoin Tracker (`btc`)
Real-time Bitcoin price and halving block metrics.
```text
.install btc https://github.com/Thruqe/whatsrook-externals/releases/latest/download/btc
```
*Short command:*
```text
.install btc
```

### 10. Forex & Market Rates (`markets`)
Forex Factory market rates (Forex currencies, Commodities, Indices, and Crypto).
```text
.install markets https://github.com/Thruqe/whatsrook-externals/releases/latest/download/markets
```
*Short command:*
```text
.install markets
```

### 11. AP News Headlines (`news`)
Latest top news headlines by country from AP News.
```text
.install news https://github.com/Thruqe/whatsrook-externals/releases/latest/download/news
```
*Short command:*
```text
.install news
```

### 12. WABetaInfo Updates (`wabeta`)
Latest WhatsApp beta features and breakdowns from WABetaInfo.
```text
.install wabeta https://github.com/Thruqe/whatsrook-externals/releases/latest/download/wabeta
```
*Short command:*
```text
.install wabeta
```

### 13. Why.com AI Deep Search (`why`)
AI-powered knowledge reasoning and deep-search exploration from why.com.
```text
.install why https://github.com/Thruqe/whatsrook-externals/releases/latest/download/why
```
*Short command:*
```text
.install why
```

### 14. Webpage Screenshot (`ss`)
Capture full website screenshots via high-resolution rendering engines.
```text
.install ss https://github.com/Thruqe/whatsrook-externals/releases/latest/download/ss
```
*Short command:*
```text
.install ss
```

### 15. Text-To-Speech (`tts`)
Convert text to natural speech audio via Google TTS with multilingual support.
```text
.install tts https://github.com/Thruqe/whatsrook-externals/releases/latest/download/tts
```
*Short command:*
```text
.install tts
```

### 16. QR Code Generator (`qrcode`)
Generate high-resolution QR code PNG images from any text or URL.
```text
.install qrcode https://github.com/Thruqe/whatsrook-externals/releases/latest/download/qrcode
```
*Short command:*
```text
.install qrcode
```

### 17. Fancy Font Styler (`fancy`)
Transform plain text into over 20+ decorative Unicode font styles (Fraktur, Bold, Italic, Small Caps, Double Struck, etc.).
```text
.install fancy https://github.com/Thruqe/whatsrook-externals/releases/latest/download/fancy
```
*Short command:*
```text
.install fancy
```

### 18. Font Styler (`font`)
Switch or apply decorative Unicode fonts directly.
```text
.install font https://github.com/Thruqe/whatsrook-externals/releases/latest/download/font
```
*Short command:*
```text
.install font
```

### 19. Fonts Style Directory (`fonts`)
List all numbered font styles and visual previews.
```text
.install fonts https://github.com/Thruqe/whatsrook-externals/releases/latest/download/fonts
```
*Short command:*
```text
.install fonts
```

### 20. GitHub Repository Explorer (`git`)
Download repository .zip archives, view commits, branches, releases, and user profiles.
```text
.install git https://github.com/Thruqe/whatsrook-externals/releases/latest/download/git
```
*Short command:*
```text
.install git
```

### 21. MP4 URL Video Downloader (`mp4url`)
Download and stream MP4 video files directly from web URLs.
```text
.install mp4url https://github.com/Thruqe/whatsrook-externals/releases/latest/download/mp4url
```
*Short command:*
```text
.install mp4url
```

### 22. CPU Monitor (`cpu`)
Display host processor model, architecture, active cores, and load averages.
```text
.install cpu https://github.com/Thruqe/whatsrook-externals/releases/latest/download/cpu
```
*Short command:*
```text
.install cpu
```

### 23. System Memory Monitor (`memory`)
Display host total RAM, used memory percentage, and available system resources.
```text
.install memory https://github.com/Thruqe/whatsrook-externals/releases/latest/download/memory
```
*Short command:*
```text
.install memory
```

---

## 🚀 Quick Download & Platform Matrix

Pre-compiled binary releases are available for all major architectures and operating systems on each release tag:

| Platform / Architecture | Suffix | All-In-One Bundle |
| :--- | :--- | :--- |
| **Linux x86_64** (Ubuntu, Debian, Fedora, Arch) | `linux-amd64` | [`whatsrook-externals-linux-amd64.tar.gz`](https://github.com/Thruqe/whatsrook-externals/releases/latest/download/whatsrook-externals-linux-amd64.tar.gz) |
| **Linux ARM64 / aarch64** (Raspberry Pi, ARM Servers, Termux) | `linux-arm64` | [`whatsrook-externals-linux-arm64.tar.gz`](https://github.com/Thruqe/whatsrook-externals/releases/latest/download/whatsrook-externals-linux-arm64.tar.gz) |
| **Linux Static musl x86_64** (Alpine, Docker containers) | `linux-musl-amd64` | [`whatsrook-externals-linux-musl-amd64.tar.gz`](https://github.com/Thruqe/whatsrook-externals/releases/latest/download/whatsrook-externals-linux-musl-amd64.tar.gz) |
| **macOS Apple Silicon** (M1/M2/M3/M4) | `darwin-arm64` | [`whatsrook-externals-darwin-arm64.tar.gz`](https://github.com/Thruqe/whatsrook-externals/releases/latest/download/whatsrook-externals-darwin-arm64.tar.gz) |
| **macOS Intel** (x86_64) | `darwin-amd64` | [`whatsrook-externals-darwin-amd64.tar.gz`](https://github.com/Thruqe/whatsrook-externals/releases/latest/download/whatsrook-externals-darwin-amd64.tar.gz) |
| **Windows x64** | `windows-amd64.exe` | [`whatsrook-externals-windows-amd64.zip`](https://github.com/Thruqe/whatsrook-externals/releases/latest/download/whatsrook-externals-windows-amd64.zip) |

---

## 🛠️ Usage with WhatsRook

### 1. Installing Plugins
From WhatsApp (restricted to bot owner/sudoers):
```text
.install weather
.install all
```
Or with clean universal URL or local path:
```text
.install weather https://github.com/Thruqe/whatsrook-externals/releases/latest/download/weather
.install weather /opt/plugins/weather
```

### 2. Running Commands
```text
.weather Tokyo
.calc sqrt(144) + 2^3
.urban drip
.btc
.markets Gold/USD
.news nigeria
.why what makes stars shine?
```

### 3. Listing & Managing Plugins
```text
.plist
.uninstall weather
.uninstall all
```

---

## 🏗️ Building from Source

Ensure you have the Rust toolchain installed:

```bash
git clone https://github.com/Thruqe/whatsrook-externals.git
cd whatsrook-externals

# Build all plugins in release mode
cargo build --release --workspace
```

The compiled binaries will be placed in `target/release/`:
```text
target/release/weather
target/release/urban
target/release/shorturl
target/release/calc
target/release/fact
target/release/quotes
target/release/joke
target/release/rizz
target/release/btc
target/release/markets
target/release/news
target/release/wabeta
target/release/why
target/release/ss
target/release/tts
target/release/qrcode
target/release/fancy
target/release/git
target/release/mp4url
target/release/cpu
target/release/memory
```

---

## 🧑‍💻 Writing Custom Plugins with `whatsrook-sdk`

You can easily create your own external plugin using the included `whatsrook-sdk`:

```toml
[dependencies]
whatsrook-sdk = { path = "../whatsrook-sdk" }
```

```rust
use whatsrook_sdk::{respond, Request};

fn main() {
    let req = Request::load();
    let query = req.query();

    if query.is_empty() {
        respond("Usage: .mycommand <text>");
        return;
    }

    respond(format!("Echo: {}", query));
}
```

---

## 📄 License

MIT License. Copyright (c) 2026 Thruqe.
