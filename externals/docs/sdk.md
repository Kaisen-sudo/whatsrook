# Plugin Development with `whatsrook-sdk`

The [`whatsrook-sdk`](https://crates.io/crates/whatsrook-sdk) crate makes it straightforward to author custom external plugins for WhatsRook using Rust.

---

## Architecture: IPC Over Stdin/Stdout

When WhatsRook triggers an external plugin, it spawns the binary as a child process and writes a JSON payload to its `stdin`:

```json
{
  "command": "mycommand",
  "query": "hello world",
  "sender": "1234567890@s.whatsapp.net",
  "chat": "1234567890@s.whatsapp.net",
  "is_group": false,
  "bot_name": "WhatsRook",
  "prefix": ".",
  "raw_args": "hello world"
}
```

The plugin processes the input and writes a response to `stdout`. Simple plugins use plain-text output; live/media plugins write structured JSON action frames.

```text
WhatsRook  ──stdin──▶  plugin (reads Request JSON)
plugin     ──stdout─▶  WhatsRook (reads text or Action frames)
WhatsRook  ──stdin──▶  plugin (sends Ack for live actions)
```

---

## Quick Tutorial: Creating a Custom Plugin

### 1. Initialize a new crate

```bash
cargo new --bin myplugin
cd myplugin
```

### 2. Add the SDK to `Cargo.toml`

```toml
[dependencies]
whatsrook-sdk = "0.1"
```

### 3. Write plugin logic

```rust
use whatsrook_sdk::{respond, Request};

fn main() {
    let req = Request::load();
    let query = req.query();

    if query.is_empty() {
        respond(format!("Usage: {}mycommand <query>", req.prefix()));
        return;
    }

    respond(format!("Received: {}", query));
}
```

### 4. Build and install

```bash
cargo build --release
.install mycommand /path/to/target/release/myplugin
```

---

## Action Reference

| Action | Helper | Description |
| :--- | :--- | :--- |
| `reply` | `send_reply_live(text)` | Send text, returns `msg_id` for edits |
| `edit` | `send_edit_live(id, text)` | In-place message edit |
| `react` | `send_react(emoji)` | Emoji reaction on the triggering message |
| `delete` | `send_delete(id)` | Revoke a message for everyone |
| `send_image` | `send_image(data, caption)` | Image from URL or base64 |
| `send_audio` | `send_audio(data, ptt)` | Audio or voice note |
| `send_video` | `send_video(data, caption)` | Video; `send_gif` for looping GIF |
| `send_document` | `send_document(data, name, caption)` | File attachment |
| `send_sticker` | `send_sticker(data)` | WebP sticker |
| `poll` | `send_poll(question, options)` | Interactive poll |
| `loader` | `send_loader(text)` | Typing / processing indicator |
| `done` | `send_done()` | End the live session |

For simple single-reply plugins, use `respond(text)` — no JSON framing needed.

---

## Full SDK Documentation

→ [docs.rs/whatsrook-sdk](https://docs.rs/whatsrook-sdk)
