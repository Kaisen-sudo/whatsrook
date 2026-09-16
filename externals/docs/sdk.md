# Plugin Development with `whatsrook-sdk`

The `whatsrook-sdk` crate makes it straightforward to author custom external plugins for WhatsRook using Rust.

---

## Architecture: IPC Over Stdin/Stdout

When WhatsRook triggers an external plugin, it starts the executable and writes a JSON payload to its `stdin`:

```json
{
  "command": "mycommand",
  "query": "hello world",
  "sender": "1234567890@s.whatsapp.net",
  "chat": "1234567890@s.whatsapp.net",
  "is_group": false,
  "raw_message": { ... }
}
```

The plugin processes the input and writes a response payload to `stdout`:

```json
{
  "type": "text",
  "text": "Response from my plugin!"
}
```

---

## Quick Tutorial: Creating a Custom Plugin

### 1. Initialize Crate
Create a new binary crate in your workspace:

```bash
cargo new --bin myplugin
cd myplugin
```

### 2. Configure `Cargo.toml`
Add `whatsrook-sdk` as a dependency:

```toml
[package]
name = "myplugin"
version = "0.1.0"
edition = "2021"

[dependencies]
whatsrook-sdk = { path = "../whatsrook-sdk" }
```

### 3. Write Plugin Logic
In `src/main.rs`:

```rust
use whatsrook_sdk::{respond, Request};

fn main() {
    // Load incoming request from stdin
    let req = Request::load();
    let query = req.query();

    if query.is_empty() {
        respond("Usage: .mycommand <query>");
        return;
    }

    // Process input and respond back to WhatsApp
    let reply = format!("Received query: {}", query);
    respond(reply);
}
```

### 4. Build and Install
Compile your plugin in release mode:

```bash
cargo build --release
```

Install it into WhatsRook via WhatsApp:
```text
.install mycommand /path/to/myplugin/target/release/myplugin
```
