use serde::{Deserialize, Serialize};
use std::io::{self, BufRead, IsTerminal, Write};
use std::time::Duration;

pub use reqwest;
pub use reqwest::blocking::Client as HttpClient;

pub const DEFAULT_USER_AGENT: &str =
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36";

/// Incoming request payload sent by WhatsRook via stdin.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Request {
    #[serde(default)]
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub raw_args: String,
    #[serde(default)]
    pub chat: String,
    #[serde(default)]
    pub sender: String,
    /// The bot's active command prefix (e.g. "." or "/").
    #[serde(default)]
    pub prefix: String,
    /// The configured bot display name.
    #[serde(default)]
    pub bot_name: String,
    /// The WhatsApp push name (display name) of the sender.
    #[serde(default)]
    pub push_name: String,
    /// Whether the command was triggered in a group chat.
    #[serde(default)]
    pub is_group: bool,
    /// Whether the sender is a sudo user or bot owner.
    #[serde(default)]
    pub is_sudo: bool,
    /// Whether a live session is already active for this plugin in this chat.
    #[serde(default)]
    pub live_session: bool,
    /// Whether the user is requesting cancellation of the live session.
    #[serde(default)]
    pub is_cancel_request: bool,
}

impl Request {
    /// Loads the plugin request from stdin (first line as JSON), falling back to CLI args.
    pub fn load() -> Self {
        if !io::stdin().is_terminal() {
            let stdin = io::stdin();
            let mut line = String::new();
            if stdin.lock().read_line(&mut line).is_ok() && !line.trim().is_empty() {
                if let Ok(req) = serde_json::from_str::<Request>(line.trim()) {
                    if !req.command.is_empty()
                        || !req.args.is_empty()
                        || !req.raw_args.is_empty()
                        || !req.chat.is_empty()
                    {
                        return req;
                    }
                }
            }
        }

        let cli_args: Vec<String> = std::env::args().collect();
        let command = cli_args
            .first()
            .and_then(|p| std::path::Path::new(p).file_name()?.to_str())
            .unwrap_or("plugin")
            .to_string();
        let args = if cli_args.len() > 1 {
            cli_args[1..].to_vec()
        } else {
            Vec::new()
        };
        let raw_args = args.join(" ");
        Request {
            command,
            args,
            raw_args,
            chat: String::new(),
            sender: String::new(),
            prefix: String::from("."),
            bot_name: String::from("WhatsRook"),
            push_name: String::new(),
            is_group: false,
            is_sudo: false,
            live_session: false,
            is_cancel_request: false,
        }
    }

    /// Loads the plugin request from stdin reading the first line (alias for load).
    pub fn load_streaming() -> Self {
        Self::load()
    }

    /// Helper to get trimmed raw argument string or fallback to joining args.
    pub fn query(&self) -> String {
        let trimmed = self.raw_args.trim();
        if !trimmed.is_empty() {
            trimmed.to_string()
        } else {
            self.args.join(" ").trim().to_string()
        }
    }

    /// Returns the effective prefix string. Falls back to "." if empty.
    pub fn prefix(&self) -> &str {
        if self.prefix.is_empty() {
            "."
        } else {
            &self.prefix
        }
    }

    /// Returns the bot name. Falls back to "WhatsRook" if empty.
    pub fn bot_name(&self) -> &str {
        if self.bot_name.is_empty() {
            "WhatsRook"
        } else {
            &self.bot_name
        }
    }

    /// Returns the push name (sender display name). Falls back to "User" if empty.
    pub fn push_name(&self) -> &str {
        if self.push_name.is_empty() {
            "User"
        } else {
            &self.push_name
        }
    }
}

// ─── Streaming Action Protocol ───────────────────────────────────────────────

/// A single action frame written to stdout for the streaming protocol.
#[derive(Debug, Serialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum Action<'a> {
    /// Send an initial WhatsApp message. WhatsRook replies with an Ack containing the msg_id.
    Reply { text: &'a str },
    /// Edit a previously sent message by msg_id.
    Edit { msg_id: &'a str, text: &'a str },
    /// Signal end of the streaming session.
    Done,
}

/// Acknowledgment sent by WhatsRook on stdin after a `reply` action.
#[derive(Debug, Deserialize)]
pub struct Ack {
    pub ok: bool,
    pub msg_id: Option<String>,
    pub error: Option<String>,
}

/// Write a streaming action frame to stdout and flush.
pub fn send_action(action: &Action) {
    let json = serde_json::to_string(action).unwrap_or_default();
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    let _ = writeln!(handle, "{}", json);
    let _ = handle.flush();
}

/// Read one ACK line from stdin. Returns None on EOF or parse error.
pub fn await_ack() -> Option<Ack> {
    let stdin = io::stdin();
    let mut line = String::new();
    stdin.lock().read_line(&mut line).ok()?;
    serde_json::from_str(line.trim()).ok()
}

/// Send a `reply` action, await the ACK, and return the msg_id on success.
pub fn send_reply_live(text: &str) -> Option<String> {
    send_action(&Action::Reply { text });
    let ack = await_ack()?;
    if ack.ok {
        ack.msg_id
    } else {
        None
    }
}

/// Send an `edit` action (no ACK expected).
pub fn send_edit_live(msg_id: &str, text: &str) {
    send_action(&Action::Edit { msg_id, text });
}

/// Send a `done` action to gracefully end the streaming session.
pub fn send_done() {
    send_action(&Action::Done);
}

// ─── Simple (Plain Text) Protocol ────────────────────────────────────────────

/// Sends a plain text response to WhatsRook via stdout (simple protocol).
pub fn respond(output: impl AsRef<str>) {
    print!("{}", output.as_ref().trim());
}

/// Sends an error message to stderr and user via stdout, then exits.
pub fn respond_err(error_msg: impl AsRef<str>) -> ! {
    eprintln!("{}", error_msg.as_ref());
    print!("{}", error_msg.as_ref().trim());
    std::process::exit(1);
}

// ─── HTTP Client ──────────────────────────────────────────────────────────────

/// Constructs a preconfigured blocking HTTP client with timeouts and standard headers.
pub fn create_http_client(timeout_secs: u64) -> reqwest::blocking::Client {
    reqwest::blocking::Client::builder()
        .user_agent(DEFAULT_USER_AGENT)
        .timeout(Duration::from_secs(timeout_secs))
        .build()
        .unwrap_or_default()
}
