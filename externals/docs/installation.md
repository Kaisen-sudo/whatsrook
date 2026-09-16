# Installation Guide

WhatsRook external plugins can be installed dynamically through WhatsApp chats, downloaded manually as standalone binaries, or compiled from source code.

---

## 1. Dynamic Installation via WhatsApp (Recommended)

When operating WhatsRook, bot owners and sudoers can install plugins directly through chat commands.

### Automatic Platform Detection
WhatsRook automatically detects your operating system and CPU architecture, resolving the appropriate binary download link automatically:

```text
.install <plugin-name>
```

**Examples:**
```text
.install weather
.install calc
.install btc
.install news
```

### Install All Official Plugins
To download and configure all official external plugins in one step:

```text
.install all
```

### Custom URL or Local Binary
You can also install custom plugins from arbitrary URLs or local filesystem paths:

```text
.install myplugin https://example.com/downloads/myplugin
.install myplugin /opt/whatsrook/plugins/myplugin
```

---

## 2. Managing Plugins

### List Active Plugins
View all currently registered external plugins:
```text
.plist
```

### Uninstalling Plugins
Remove a specific plugin or all plugins:
```text
.uninstall weather
.uninstall all
```

---

## 3. Prebuilt Binary Downloads

All release assets are published automatically on the [Releases Page](https://github.com/Thruqe/whatsrook-externals/releases).

| Platform / Architecture | Archive Target | Package Name |
| :--- | :--- | :--- |
| **Linux x86_64** (Ubuntu, Debian, Fedora, Arch) | `linux-amd64` | `whatsrook-externals-linux-amd64.tar.gz` |
| **Linux ARM64 / aarch64** (Raspberry Pi, ARM Cloud, Termux) | `linux-arm64` | `whatsrook-externals-linux-arm64.tar.gz` |
| **Linux musl x86_64** (Alpine Linux, Docker containers) | `linux-musl-amd64` | `whatsrook-externals-linux-musl-amd64.tar.gz` |
| **macOS Apple Silicon** (M1, M2, M3, M4) | `darwin-arm64` | `whatsrook-externals-darwin-arm64.tar.gz` |
| **macOS Intel** (x86_64) | `darwin-amd64` | `whatsrook-externals-darwin-amd64.tar.gz` |
| **Windows x64** | `windows-amd64` | `whatsrook-externals-windows-amd64.zip` |

### Unpacking Archives
For Linux/macOS:
```bash
tar -xzvf whatsrook-externals-linux-amd64.tar.gz -C ~/.whatsrook/plugins/
```

For Windows:
```powershell
Expand-Archive -Path whatsrook-externals-windows-amd64.zip -DestinationPath ~\.whatsrook\plugins
```

---

## 4. Building from Source

To compile all external plugins from source, ensure you have a standard Rust toolchain (Rust 1.75+ recommended):

```bash
# Clone the repository
git clone https://github.com/Thruqe/whatsrook-externals.git
cd whatsrook-externals

# Build all workspace crates in release mode
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
target/release/font
target/release/fonts
target/release/git
target/release/mp4url
target/release/cpu
target/release/memory
target/release/captcha
target/release/sticker
target/release/media
```

Copy the binaries to your WhatsRook plugin directory or install them locally using:
```text
.install weather /path/to/whatsrook-externals/target/release/weather
```
