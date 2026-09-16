# WhatsRook External Plugins

Welcome to the official documentation for **WhatsRook External Plugins**, the high-performance standalone extension suite for [WhatsRook](https://github.com/Thruqe/whatsrook).

---

## Overview

External plugins are modular, isolated executables built in Rust. They communicate with the host WhatsRook instance via structured JSON over standard I/O (`stdin` / `stdout`).

### Key Advantages

- **Zero-Downtime Management**: Install, update, or remove plugins live inside WhatsApp chats without recompiling or rebooting the bot.
- **Process Isolation**: Each command execution runs in its own sandboxed process. A plugin crash or panicking task will never affect core bot operations.
- **Blazing Fast**: Native Rust binaries deliver sub-millisecond start times and lightweight memory usage.
- **Multi-Platform**: Prebuilt releases available for Linux (x86_64, aarch64, musl), macOS (Apple Silicon & Intel), and Windows.
- **Developer-Friendly**: Author custom plugins in minutes using the provided `whatsrook-sdk`.

---

## Quick Navigation

- [**Installation Guide**](installation.md) - WhatsApp 1-click install, prebuilt archives, manual binary deployment, and building from source.
- [**Plugin Reference**](plugins.md) - Full catalog, descriptions, and examples for all 26 official plugins.
- [**SDK & Development Guide**](sdk.md) - Step-by-step tutorial on building and distributing your own plugins.
