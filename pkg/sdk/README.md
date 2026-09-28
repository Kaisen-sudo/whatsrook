![whatsrook](./assets/logo.svg)

_`whatsrook-sdk` is a Rust library for building external plugins that run inside [WhatsRook](https://github.com/thruqe/whatsapp-go) — with full access to the WhatsApp action protocol._

[![crates.io](https://img.shields.io/crates/v/whatsrook-sdk)](https://crates.io/crates/whatsrook-sdk)
[![docs.rs](https://img.shields.io/docsrs/whatsrook-sdk)](https://docs.rs/whatsrook-sdk)
[![CI](https://github.com/thruqe/whatsapp-go/actions/workflows/sdk-ci.yml/badge.svg?branch=master)](https://github.com/thruqe/whatsapp-go/actions/workflows/sdk-ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

## Usage

Add `whatsrook-sdk` to your `Cargo.toml`:

```toml
[dependencies]
whatsrook-sdk = "0.1"
```

External plugins are standalone binaries. WhatsRook spawns them as child processes, writes a JSON request payload to `stdin`, and reads newline-delimited JSON action frames from `stdout`. See [How It Works](#how-it-works) for the full protocol, and the [Plugin Development Guide](../../docs/EXTERNAL_PLUGIN.md) for installation and deployment.

## Features

- **Request Parsing**: `stdin` JSON deserialization with argument indexing (`arg`, `arg_as`), subcommand routing (`subcommand`), and flag evaluation (`flag`, `flag_value`).
- **Complete Action Protocol**: Text, image, audio, video, document, sticker, poll, reactions, and live in-place edits.
- **Fluent Action Builders**: Construct and chain actions with `.with_caption()`, `.as_gif()`, `.with_mimetype()`, and `.send()`.
- **WhatsApp Message Formatting**: Markdown text decorators (`bold`, `italic`, `quote`, `code_block`) and fluent [`MessageBuilder`](#message-formatting).
- **Media Utilities**: Pure-Rust RFC 4648 Base64 encoding/decoding and disk-to-data-URL helpers (`read_file_as_data_url`).
- **Live Session Support**: Send messages, receive message IDs via stdin ACK, and update them in real time.
- **Simple-Mode Output**: Single-reply plugins with zero boilerplate (`respond(...)`).
- **Preconfigured HTTP Client**: Browser-like TLS client (`create_http_client`).
- **CLI Development Fallback**: Test plugins directly from the terminal without a running WhatsApp connection.

## How It Works

When a WhatsApp user triggers a plugin command, WhatsRook:

1. Spawns the plugin binary.
2. Writes one JSON line to `stdin` — the [`Request`](https://docs.rs/whatsrook-sdk/latest/whatsrook_sdk/struct.Request.html) payload.
3. Reads newline-delimited JSON action frames from `stdout`.
4. For frames that return a message ID (e.g. `reply`), writes an acknowledgement back to `stdin`.

```text
WhatsRook  ──stdin──▶  plugin (reads Request JSON)
plugin     ──stdout─▶  WhatsRook (reads Action frames)
WhatsRook  ──stdin──▶  plugin (reads Ack, for live actions)
```

## Quick Start

```rust
use whatsrook_sdk::{respond, Request};

fn main() {
    let req = Request::load();
    let query = req.query();

    if query.is_empty() {
        respond("Usage: .hello <name>");
        return;
    }

    respond(format!("Hello, {}! 👋", query));
}
```

## Argument & Flag Parsing

```rust
use whatsrook_sdk::{respond, Request};

fn main() {
    let req = Request::load();

    // Subcommand dispatch
    match req.subcommand() {
        Some("status") => respond("Server status: OK"),
        Some("count") => {
            let limit: u32 = req.arg_as(1).unwrap_or(10);
            let verbose = req.flag("verbose") || req.flag("v");
            respond(format!("Counting to {} (verbose: {})", limit, verbose));
        }
        _ => respond("Usage: .mybot <status|count>"),
    }
}
```

## Message Formatting

Format text with WhatsApp Markdown decorators or compose complex messages with `MessageBuilder`:

```rust
use whatsrook_sdk::fmt::MessageBuilder;

let msg = MessageBuilder::new()
    .header("Server Diagnostics")
    .bullet("CPU: 12% across 8 cores")
    .bullet("RAM: 3.2 GB / 16 GB")
    .newline()
    .quote("Cluster health: optimal")
    .build();

whatsrook_sdk::respond(msg);
```

## Live Session Example

```rust
use whatsrook_sdk::{
    create_http_client, respond, send_done, send_edit_live,
    send_poll, send_react, send_reply_live, Request,
};

fn main() {
    let req = Request::load();

    // Permission check
    if req.is_group() && !req.is_admin() {
        send_react("❌");
        respond("This feature is for group admins only.");
        return;
    }

    // Echo a quoted message if present
    if let Some(quoted) = req.quoted_text() {
        println!("User replied to: {}", quoted);
    }

    // React and send a poll
    send_react("🚀");
    send_poll("Which asset to track?", &["BTC", "ETH", "Gold"]);

    // Send a live message and update it in-place
    if let Some(msg_id) = send_reply_live("⏳ Initializing live tracker...") {
        for i in 1..=5 {
            std::thread::sleep(std::time::Duration::from_millis(1500));
            send_edit_live(&msg_id, &format!("📈 Tracker tick #{}...", i));
        }
    }

    send_done();
}
```

## Action Reference

| Action | Helper | Description |
| :--- | :--- | :--- |
| `reply` | `send_reply_live(text)` | Send text, returns `msg_id` for edits |
| `edit` | `send_edit_live(id, text)` | In-place message edit |
| `react` | `send_react(emoji)` / `send_react_to(id, emoji)` | Emoji reaction |
| `delete` | `send_delete(id)` | Revoke a message for everyone |
| `send_image` | `send_image(data, caption)` | Image from URL or base64 |
| `send_audio` | `send_audio(data, ptt)` | Audio or voice note |
| `send_video` | `send_video(data, caption)` | Video; `send_gif` for looping GIF |
| `send_document` | `send_document(data, name, caption)` | File attachment |
| `send_sticker` | `send_sticker(data)` | WebP sticker |
| `poll` | `send_poll(question, options)` / `send_multi_poll` | Interactive poll |
| `loader` | `send_loader(text)` | Typing / processing indicator |
| `done` | `send_done()` | End the live session |

You can also use fluent builders directly:

```rust
use whatsrook_sdk::Action;

Action::image("https://example.com/art.png")
    .with_caption("AI Generated Artwork")
    .send();
```

For simple single-reply plugins, use `respond(text)` — plain text written to `stdout`, no JSON framing needed.


## Supported Targets

| Target | Description |
| :--- | :--- |
| `x86_64-unknown-linux-musl` | Linux AMD64 — static MUSL binary |
| `aarch64-unknown-linux-musl` | Linux ARM64 / Android Termux — static MUSL |
| `aarch64-apple-darwin` | macOS Apple Silicon |
| `x86_64-apple-darwin` | macOS Intel |
| `x86_64-pc-windows-msvc` | Windows x64 |

## Contributions

If you want to help make this project better, please take the time to read the [contribution guide](./CONTRIBUTING.md) and [fork](https://github.com/Thruqe/whatsrook-sdk/fork) this repository. Then open a pull request with your changes.

## Acknowledgements

whatsrook-sdk is part of the [WhatsRook](https://github.com/Thruqe/whatsrook) ecosystem. The IPC protocol it implements is defined by the WhatsRook runtime and its [external plugin engine](https://github.com/Thruqe/whatsrook/blob/master/docs/EXTERNAL_PLUGIN.md).

## Licensing

This project is open source, see the [LICENSE](./LICENSE) file for full details.
