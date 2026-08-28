# WhatsRook External Plugins 🔌

Official suite of standalone, high-performance external plugins for **[WhatsRook](https://github.com/Thruqe/whatsrook)** written in Rust.

External plugins run as isolated child processes, communicating with WhatsRook via JSON over `stdin` and returning replies over `stdout`. They do not require recompilation or modifications to the WhatsRook codebase and can be installed or uninstalled dynamically at runtime via WhatsApp commands.

---

## 📦 Available Plugins

| Command | Category | Description | Direct WhatsApp Installation (Linux AMD64) |
| :--- | :--- | :--- | :--- |
| **`weather`** | Utilities | Real-time weather forecast for any city or town | `.install weather https://github.com/Thruqe/whatsrook-externals/releases/latest/download/weather-linux-amd64` |
| **`urban`** | Lookup | Urban Dictionary word/slang definition lookup | `.install urban https://github.com/Thruqe/whatsrook-externals/releases/latest/download/urban-linux-amd64` |
| **`shorturl`** | Utilities | Shortens URLs using TinyURL and is.gd | `.install shorturl https://github.com/Thruqe/whatsrook-externals/releases/latest/download/shorturl-linux-amd64` |
| **`calc`** | Math | Mathematical expression evaluator (`+`, `-`, `*`, `/`, `^`, `sqrt`, `sin`, `cos`, `pi`, `e`, etc.) | `.install calc https://github.com/Thruqe/whatsrook-externals/releases/latest/download/calc-linux-amd64` |
| **`fact`** | Fun | Interesting random facts from public APIs | `.install fact https://github.com/Thruqe/whatsrook-externals/releases/latest/download/fact-linux-amd64` |
| **`quotes`** | Fun | Inspirational quotes and authors | `.install quotes https://github.com/Thruqe/whatsrook-externals/releases/latest/download/quotes-linux-amd64` |
| **`joke`** | Fun | Clean jokes and funny punchlines | `.install joke https://github.com/Thruqe/whatsrook-externals/releases/latest/download/joke-linux-amd64` |
| **`rizz`** | Fun | Smooth pickup lines & rizz | `.install rizz https://github.com/Thruqe/whatsrook-externals/releases/latest/download/rizz-linux-amd64` |
| **`btc`** | Finance | Real-time Bitcoin price and halving block metrics | `.install btc https://github.com/Thruqe/whatsrook-externals/releases/latest/download/btc-linux-amd64` |
| **`markets`** | Finance | Forex Factory market rates (Forex, Commodities, Indices, Crypto) | `.install markets https://github.com/Thruqe/whatsrook-externals/releases/latest/download/markets-linux-amd64` |
| **`news`** | News | Latest top news headlines by country from AP News | `.install news https://github.com/Thruqe/whatsrook-externals/releases/latest/download/news-linux-amd64` |
| **`wabeta`** | News | Latest WhatsApp beta features and breakdowns from WABetaInfo | `.install wabeta https://github.com/Thruqe/whatsrook-externals/releases/latest/download/wabeta-linux-amd64` |
| **`why`** | AI / Search | AI-powered knowledge reasoning and deep-search from why.com | `.install why https://github.com/Thruqe/whatsrook-externals/releases/latest/download/why-linux-amd64` |

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

To install a specific binary on your system, substitute the architecture suffix in the release URL:

```text
https://github.com/Thruqe/whatsrook-externals/releases/latest/download/<command>-<platform-suffix>
```

---

## 🛠️ Usage with WhatsRook

### 1. Installing a Plugin
From WhatsApp (restricted to bot owner/sudoers):
```text
.install weather https://github.com/Thruqe/whatsrook-externals/releases/latest/download/weather-linux-amd64
```
Or from a local binary on the server:
```text
.install weather /opt/plugins/weather
```

### 2. Running the Command
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
```bash
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
