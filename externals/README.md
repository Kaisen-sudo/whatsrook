# WhatsRook External Plugins

[![CI](https://github.com/Thruqe/whatsrook-externals/actions/workflows/ci.yml/badge.svg)](https://github.com/Thruqe/whatsrook-externals/actions/workflows/ci.yml)
[![Website](https://github.com/Thruqe/whatsrook-externals/actions/workflows/website.yml/badge.svg)](https://thruqe.github.io/whatsrook-externals/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

Official suite of standalone, high-performance external plugins for **[WhatsRook](https://github.com/Thruqe/whatsrook)** written in Rust.

External plugins run as isolated child processes, communicating with WhatsRook via JSON over `stdin` and returning replies over `stdout`. They do not require recompilation of WhatsRook and can be installed, updated, or removed dynamically at runtime via WhatsApp commands.

---

## Instant Installation via WhatsApp

With WhatsRook's platform-aware installer, simply run:

```text
.install <plugin>
```

To install all official plugins at once:

```text
.install all
```

For complete installation methods (prebuilt binaries, custom URLs, and building from source), see the **[Installation Guide](docs/installation.md)**.

---

## Available Plugins (26)

| Plugin | Command | Description |
| :--- | :--- | :--- |
| **weather** | `.weather <city>` | Real-time weather forecasts |
| **calc** | `.calc <expr>` | Mathematical and scientific calculation |
| **urban** | `.urban <term>` | Urban Dictionary slang definitions |
| **shorturl** | `.shorturl <url>` | Fast URL shortening |
| **btc** | `.btc` | Live Bitcoin price and metrics |
| **markets** | `.markets <pair>` | Forex, crypto, and stock market rates |
| **news** | `.news [country]` | AP News top headlines |
| **wabeta** | `.wabeta` | WhatsApp beta news and feature breakdowns |
| **why** | `.why <prompt>` | AI deep search and reasoning from Why.com |
| **ss** | `.ss <url>` | Full webpage screenshot capture |
| **tts** | `.tts <lang> <text>` | Google Text-To-Speech audio |
| **qrcode** | `.qrcode <data>` | High-resolution QR code generator |
| **fancy** | `.fancy <text>` | 20+ decorative Unicode fonts |
| **git** | `.git <owner/repo>` | GitHub repository explorer and ZIP downloads |
| **cpu** | `.cpu` | Host CPU specifications and load |
| **memory** | `.memory` | Host RAM and memory usage |
| **media** | `.media` | Video/audio converter, trimmer, and extractor |
| **sticker** | `.sticker` | Custom WebP sticker creator |
| **captcha** | `.captcha` | Dial verification code video generator |
| *...and more* | | [View complete plugin list & usage](docs/plugins.md) |

---

## Documentation

Comprehensive documentation is available in the **[`docs/`](docs/)** directory and on the **[Documentation Website](https://thruqe.github.io/whatsrook-externals/)**:

- 📖 **[Installation Guide](docs/installation.md)** - 1-Click install, prebuilt archives, manual binary setups, and compilation.
- 🔌 **[Plugin Reference](docs/plugins.md)** - Detailed commands, syntax, and examples for all 26 plugins.
- 🛠️ **[SDK & Plugin Development](docs/sdk.md)** - Writing your own external plugins in Rust using `whatsrook-sdk`.

---

## Building from Source

```bash
git clone https://github.com/Thruqe/whatsrook-externals.git
cd whatsrook-externals
cargo build --release --workspace
```

---

## License

MIT License. Copyright (c) 2026 Thruqe.
