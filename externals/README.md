![whatsrook](./assets/logo.svg)

_`whatsrook-externals` is the official suite of standalone external plugins for [WhatsRook](https://github.com/Thruqe/whatsrook), built in Rust using [`whatsrook-sdk`](https://crates.io/crates/whatsrook-sdk)._

[![CI](https://github.com/Thruqe/whatsrook-externals/actions/workflows/ci.yml/badge.svg?branch=master)](https://github.com/Thruqe/whatsrook-externals/actions/workflows/ci.yml)
[![Website](https://img.shields.io/website?url=https%3A%2F%2Fthruqe.github.io%2Fwhatsrook-externals%2F)](https://thruqe.github.io/whatsrook-externals/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

## Installation

With WhatsRook's platform-aware installer, run from any WhatsApp chat:

```text
.install <plugin>
```

To install every official plugin in one go:

```text
.install all
```

For prebuilt binaries, custom URLs, and building from source, see the [Installation Guide](https://thruqe.github.io/whatsrook-externals/installation/).

## Plugins

| Command | Plugin | Category | Description |
| :--- | :--- | :--- | :--- |
| `.weather` | `weather` | Utility | Real-time weather forecasts by city or coordinates |
| `.urban` | `urban` | Reference | Urban Dictionary definitions, examples, and votes |
| `.shorturl` | `shorturl` | Utility | URL shortener via TinyURL and is.gd |
| `.calc` | `calc` | Math | High-precision mathematical and scientific expression evaluator |
| `.fact` | `fact` | Fun | Random interesting facts with offline fallbacks |
| `.quotes` | `quotes` | Fun | Inspirational quotes and attributed authors |
| `.joke` | `joke` | Fun | Clean, witty jokes and punchlines |
| `.rizz` | `rizz` | Fun | Modern pickup lines and charismatic phrases |
| `.btc` | `btc` | Finance | Live Bitcoin price, 24h change, and halving countdown |
| `.markets` | `markets` | Finance | Forex, crypto, commodity, and index rates |
| `.news` | `news` | Media | AP News top headlines by country code |
| `.wabeta` | `wabeta` | WhatsApp | WhatsApp beta feature leaks from WABetaInfo |
| `.why` | `why` | AI | AI-powered deep search and reasoning from Why.com |
| `.ss` | `ss` | Utility | Full-page webpage screenshot capture |
| `.tts` | `tts` | Media | Google Text-to-Speech audio notes |
| `.qrcode` | `qrcode` | Utility | High-resolution QR code generator |
| `.fancy` | `fancy` | Styling | Converts text into 20+ decorative Unicode fonts |
| `.font` | `font` | Styling | Renders text in a specific numbered Unicode font |
| `.fonts` | `fonts` | Styling | Directory and visual samples of all font styles |
| `.git` | `git` | Developer | GitHub repo explorer, commits, releases, and ZIP downloads |
| `.mp4url` | `mp4url` | Media | Downloads and streams video from direct MP4 URLs |
| `.cpu` | `cpu` | System | Host CPU model, cores, threads, and load |
| `.memory` | `memory` | System | Host RAM usage and available memory |
| `.captcha` | `captcha` | Utility | Animated dial verification code video captchas |
| `.sticker` | `sticker` | Media | Converts media to a letterboxed 512×512 WebP sticker |
| `.circle` | `circle` | Media | Converts media to a circular-masked WebP sticker |
| `.crop` | `crop` | Media | Converts media to a square-cropped WebP sticker |
| `.take` | `take` | Media | Re-packs sticker metadata — author and pack name |
| `.media` | `media` | Media | Video/audio converter, audio extractor, and trimmer |

Full usage examples and command reference: [Plugin Catalog](https://thruqe.github.io/whatsrook-externals/plugins/)

## Building from Source

```bash
git clone https://github.com/Thruqe/whatsrook-externals.git
cd whatsrook-externals
cargo build --release --workspace
```

Requires Rust stable (1.85+) and `ffmpeg` on `PATH` for media and sticker plugins.

## Contributions

If you want to help make this project better, please read the [contribution guide](https://thruqe.github.io/whatsrook-externals/) and [fork](https://github.com/Thruqe/whatsrook-externals/fork) this repository. Then open a pull request with your changes.

## Acknowledgements

whatsrook-externals is part of the [WhatsRook](https://github.com/Thruqe/whatsrook) ecosystem. All plugins are built with [`whatsrook-sdk`](https://crates.io/crates/whatsrook-sdk) — the official Rust SDK for external plugin development.

## Licensing

This project is open source, see the [LICENSE](./LICENSE) file for full details.
