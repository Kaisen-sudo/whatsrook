# Troubleshooting & Diagnostics

This guide provides solutions for common issues encountered when installing, configuring, or running WhatsRook external plugins.

---

## Common Issues & Fixes

### 1. `Permission denied (os error 13)`
On Linux and macOS, newly downloaded or extracted binaries may lack the executable permission bit.

???+ failure "Symptom"
    WhatsRook logs indicate: `fork/exec /home/user/.whatsrook/plugins/weather: permission denied`

**Solution**:
Run `chmod +x` across the plugins directory:

```bash
chmod +x ~/.whatsrook/plugins/*
```

---

### 2. Media / Sticker Commands Fail (`ffmpeg not found`)
Plugins such as `.sticker`, `.circle`, `.crop`, `.take`, `.media`, and `.captcha` invoke `ffmpeg` under the hood.

???+ failure "Symptom"
    Command replies with `ffmpeg executable not found` or exits with non-zero status.

**Solution**:
Verify that `ffmpeg` is installed and reachable on system `$PATH`:

```bash
ffmpeg -version
```

If missing, install via your package manager:

=== "Debian / Ubuntu"
    ```bash
    sudo apt update && sudo apt install -y ffmpeg
    ```

=== "Alpine Linux"
    ```bash
    apk add --no-cache ffmpeg
    ```

=== "macOS (Homebrew)"
    ```bash
    brew install ffmpeg
    ```

=== "Windows"
    ```powershell
    winget install Gyan.FFmpeg
    ```

---

### 3. Command Timeout Errors
One-shot commands that do not produce output within **30 seconds** are terminated by WhatsRook to prevent stalled worker threads.

???+ failure "Symptom"
    Command hangs and produces no response, or logs report `plugin process killed after 30s timeout`.

**Causes & Solutions**:
- **Slow Remote APIs**: Ensure your plugin uses connection and read timeouts on HTTP requests. In Rust:
  ```rust
  let client = whatsrook_sdk::create_http_client(10); // 10s timeout
  ```
- **Heavy Computations**: For tasks that take more than a few seconds, use the **Live Session** workflow (`send_reply_live` + `send_edit_live`). Live sessions allow up to **5 minutes** of active runtime.

---

### 4. Delayed or Missing Responses (Stdout Buffering)
If a plugin prints to `stdout` without flushing, messages may sit in OS buffers indefinitely.

???+ warning "Flush Standard Output"
    Always flush standard output after printing JSON action frames:
    ```rust
    use std::io::{self, Write};
    println!("{}", json_action);
    io::stdout().flush().unwrap();
    ```
    *Note: All high-level helpers in `whatsrook-sdk` flush automatically.*

---

## Standalone Plugin Debugging

You can test any external plugin directly in your terminal without starting WhatsRook.

### Simulate Command Execution

Feed a mock `Request` JSON payload directly into the plugin executable over standard input:

=== "Linux / macOS"
    ```bash
    echo '{"command":"weather","query":"Madrid","prefix":"."}' | ~/.whatsrook/plugins/weather
    ```

=== "Windows (PowerShell)"
    ```powershell
    '{"command":"calc","query":"2^16","prefix":"."}' | .\target\release\calc.exe
    ```

### Expected Behavior
The plugin should immediately output valid JSON action frames or plaintext and exit with status code `0`.

---

## Managing Live Sessions

If a long-running live session plugin (such as `.btc` ticker) is active in a chat and you wish to stop it prematurely:

```text
.cancel
```

WhatsRook will signal the active process with `is_cancel_request: true` or terminate the process if it fails to exit promptly.

---

## Frequently Asked Questions (FAQ)

### Can I write plugins in languages other than Rust?
**Yes.** The WhatsRook external plugin protocol relies solely on newline-delimited JSON over `stdin` and `stdout`. You can write plugins in Python, Go, Node.js, C++, or Bash as long as the binary can read JSON from `stdin` and output valid action frames to `stdout`.

### Where are plugin binaries located?
By default, plugins are stored in `~/.whatsrook/plugins/`. You can change this path by exporting the `WHATSROOK_DATA_DIR` environment variable before running the bot.

### How do I update a plugin to the latest version?
Simply re-run `.install <plugin>` or `.install all` inside WhatsApp. WhatsRook overwrites the existing binary in-place with the latest release.
