# WhatsRook External Plugins 🔌

Official suite of standalone, high-performance external plugins for **[WhatsRook](https://github.com/Thruqe/whatsrook)** written in Rust.

External plugins run as isolated child processes, communicating with WhatsRook via JSON over `stdin` and returning replies over `stdout`. They do not require recompilation or modifications to the WhatsRook codebase and can be installed or uninstalled dynamically at runtime via WhatsApp commands.

---

## ⚡ Instant 1-Click WhatsApp Installation

With WhatsRook's platform-aware installer, simply run:

```text
.install <command>
```

WhatsRook will automatically detect your server's operating system (Linux, macOS, Windows) and architecture (AMD64, ARM64) and download the matching binary from this repository.

To install all official plugins at once:
```text
.install all
```

---

## 📦 Available Plugins & Installation Commands

Click the copy button on any block below to install directly from WhatsApp:

### 1. Weather (`weather`)
Real-time weather forecast for any city or town.
```text
.install weather
```
*Direct binary link (Linux x86_64):*
```text
https://github.com/Thruqe/whatsrook-externals/releases/latest/download/weather-linux-amd64
```

### 2. Urban Dictionary (`urban`)
Urban Dictionary slang and definition lookup.
```text
.install urban
```
*Direct binary link (Linux x86_64):*
```text
https://github.com/Thruqe/whatsrook-externals/releases/latest/download/urban-linux-amd64
```

### 3. URL Shortener (`shorturl`)
Shortens long URLs using TinyURL and is.gd.
```text
.install shorturl
```
*Direct binary link (Linux x86_64):*
```text
https://github.com/Thruqe/whatsrook-externals/releases/latest/download/shorturl-linux-amd64
```

### 4. Calculator & Math (`calc`)
Mathematical expression evaluator (`+`, `-`, `*`, `/`, `%`, `^`, `sqrt`, `sin`, `cos`, `tan`, `log`, `ln`, `pi`, `e`, etc.).
```text
.install calc
```
*Direct binary link (Linux x86_64):*
```text
https://github.com/Thruqe/whatsrook-externals/releases/latest/download/calc-linux-amd64
```

### 5. Random Facts (`fact`)
Interesting random facts from public APIs with offline fallbacks.
```text
.install fact
```
*Direct binary link (Linux x86_64):*
```text
https://github.com/Thruqe/whatsrook-externals/releases/latest/download/fact-linux-amd64
```

### 6. Inspirational Quotes (`quotes`)
Inspirational quotes and authors.
```text
.install quotes
```
*Direct binary link (Linux x86_64):*
```text
https://github.com/Thruqe/whatsrook-externals/releases/latest/download/quotes-linux-amd64
```

### 7. Jokes (`joke`)
Clean jokes and funny punchlines.
```text
.install joke
```
*Direct binary link (Linux x86_64):*
```text
https://github.com/Thruqe/whatsrook-externals/releases/latest/download/joke-linux-amd64
```

### 8. Rizz & Pickup Lines (`rizz`)
Smooth pickup lines & rizz.
```text
.install rizz
```
*Direct binary link (Linux x86_64):*
```text
https://github.com/Thruqe/whatsrook-externals/releases/latest/download/rizz-linux-amd64
```

### 9. Bitcoin Tracker (`btc`)
Real-time Bitcoin price and halving block metrics.
```text
.install btc
```
*Direct binary link (Linux x86_64):*
```text
https://github.com/Thruqe/whatsrook-externals/releases/latest/download/btc-linux-amd64
```

### 10. Forex & Market Rates (`markets`)
Forex Factory market rates (Forex currencies, Commodities, Indices, and Crypto).
```text
.install markets
```
*Direct binary link (Linux x86_64):*
```text
https://github.com/Thruqe/whatsrook-externals/releases/latest/download/markets-linux-amd64
```

### 11. AP News Headlines (`news`)
Latest top news headlines by country from AP News.
```text
.install news
```
*Direct binary link (Linux x86_64):*
```text
https://github.com/Thruqe/whatsrook-externals/releases/latest/download/news-linux-amd64
```

### 12. WABetaInfo Updates (`wabeta`)
Latest WhatsApp beta features and breakdowns from WABetaInfo.
```text
.install wabeta
```
*Direct binary link (Linux x86_64):*
```text
https://github.com/Thruqe/whatsrook-externals/releases/latest/download/wabeta-linux-amd64
```

### 13. Why.com AI Deep Search (`why`)
AI-powered knowledge reasoning and deep-search exploration from why.com.
```text
.install why
```
*Direct binary link (Linux x86_64):*
```text
https://github.com/Thruqe/whatsrook-externals/releases/latest/download/why-linux-amd64
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

To download any specific binary directly for your system:
```text
https://github.com/Thruqe/whatsrook-externals/releases/latest/download/<command>-<platform-suffix>
```

---

## 🛠️ Usage with WhatsRook

### 1. Installing Plugins
From WhatsApp (restricted to bot owner/sudoers):
```text
.install weather
.install all
```
Or with custom URL / local path:
```text
.install weather https://github.com/Thruqe/whatsrook-externals/releases/latest/download/weather-linux-amd64
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
