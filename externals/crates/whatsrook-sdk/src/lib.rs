use serde::{Deserialize, Serialize};
use std::io::{self, BufRead, IsTerminal, Write};
use std::time::Duration;

pub use reqwest;
pub use reqwest::blocking::Client as HttpClient;

pub const DEFAULT_USER_AGENT: &str =
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36";

/// Quoted message context payload.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct QuotedMessage {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub sender: String,
    #[serde(default)]
    pub text: String,
}

/// Incoming request payload sent by WhatsRook via standard input.
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
    /// Whether the sender is the primary bot owner.
    #[serde(default)]
    pub is_owner: bool,
    /// Whether the sender is a group admin.
    #[serde(default)]
    pub is_admin: bool,
    /// Whether a live session is already active for this plugin in this chat.
    #[serde(default)]
    pub live_session: bool,
    /// Whether the user is requesting cancellation of the live session.
    #[serde(default)]
    pub is_cancel_request: bool,
    /// Quoted message context if the triggering message is a reply.
    #[serde(default)]
    pub quoted_message: Option<QuotedMessage>,
    /// List of mentioned user JIDs in the triggering message.
    #[serde(default)]
    pub mentioned_jids: Vec<String>,
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
            is_owner: false,
            is_admin: false,
            live_session: false,
            is_cancel_request: false,
            quoted_message: None,
            mentioned_jids: Vec::new(),
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

    /// Returns quoted text if a quoted message is present.
    pub fn quoted_text(&self) -> Option<&str> {
        self.quoted_message.as_ref().map(|q| q.text.as_str())
    }

    /// Returns quoted sender JID if a quoted message is present.
    pub fn quoted_sender(&self) -> Option<&str> {
        self.quoted_message.as_ref().map(|q| q.sender.as_str())
    }

    /// Returns quoted message ID if a quoted message is present.
    pub fn quoted_id(&self) -> Option<&str> {
        self.quoted_message.as_ref().map(|q| q.id.as_str())
    }

    /// Returns true if invoked in a group.
    pub fn is_group(&self) -> bool {
        self.is_group
    }

    /// Returns true if sender is a sudo user or owner.
    pub fn is_sudo(&self) -> bool {
        self.is_sudo
    }

    /// Returns true if sender is the bot owner.
    pub fn is_owner(&self) -> bool {
        self.is_owner
    }

    /// Returns true if sender is a group admin.
    pub fn is_admin(&self) -> bool {
        self.is_admin
    }
}

// ─── Action Protocol (Full Capabilities) ────────────────────────────────────

/// A single action frame written to stdout for WhatsRook to execute.
#[derive(Debug, Serialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum Action<'a> {
    /// Send a text reply. WhatsRook responds with an Ack containing the sent msg_id.
    Reply { text: &'a str },
    /// Edit an existing message by msg_id.
    Edit { msg_id: &'a str, text: &'a str },
    /// React to the triggering message or a target message with an emoji.
    React {
        #[serde(skip_serializing_if = "Option::is_none")]
        msg_id: Option<&'a str>,
        emoji: &'a str,
    },
    /// Revoke/delete a message by msg_id.
    Delete { msg_id: &'a str },
    /// Send an image from base64 data or an HTTP/HTTPS URL.
    SendImage {
        data: &'a str,
        #[serde(skip_serializing_if = "Option::is_none")]
        caption: Option<&'a str>,
        #[serde(skip_serializing_if = "Option::is_none")]
        mimetype: Option<&'a str>,
    },
    /// Send audio or voice note from base64 data or an HTTP/HTTPS URL.
    SendAudio {
        data: &'a str,
        #[serde(skip_serializing_if = "Option::is_none")]
        mimetype: Option<&'a str>,
        #[serde(default)]
        ptt: bool,
    },
    /// Send video or GIF from base64 data or an HTTP/HTTPS URL.
    SendVideo {
        data: &'a str,
        #[serde(skip_serializing_if = "Option::is_none")]
        caption: Option<&'a str>,
        #[serde(skip_serializing_if = "Option::is_none")]
        mimetype: Option<&'a str>,
        #[serde(default)]
        gif_playback: bool,
    },
    /// Send a document file from base64 data or an HTTP/HTTPS URL.
    SendDocument {
        data: &'a str,
        #[serde(skip_serializing_if = "Option::is_none")]
        filename: Option<&'a str>,
        #[serde(skip_serializing_if = "Option::is_none")]
        caption: Option<&'a str>,
        #[serde(skip_serializing_if = "Option::is_none")]
        mimetype: Option<&'a str>,
    },
    /// Send a sticker from WebP base64 data or an HTTP/HTTPS URL.
    SendSticker { data: &'a str },
    /// Send an interactive poll.
    Poll {
        question: &'a str,
        options: &'a [&'a str],
        #[serde(default)]
        selectable: usize,
    },
    /// Control the typing/processing loader indicator.
    Loader {
        #[serde(skip_serializing_if = "Option::is_none")]
        text: Option<&'a str>,
    },
    /// Signal completion of the plugin session.
    Done,
}

/// Acknowledgment sent by WhatsRook on stdin following actions that return IDs.
#[derive(Debug, Deserialize)]
pub struct Ack {
    pub ok: bool,
    pub msg_id: Option<String>,
    pub error: Option<String>,
}

/// Write an action frame to stdout and flush immediately.
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

/// Send an `edit` action to update an existing message.
pub fn send_edit_live(msg_id: &str, text: &str) {
    send_action(&Action::Edit { msg_id, text });
}

/// Send an emoji reaction to the current triggering message.
pub fn send_react(emoji: &str) {
    send_action(&Action::React {
        msg_id: None,
        emoji,
    });
}

/// Send a delete/revoke action for a message ID.
pub fn send_delete(msg_id: &str) {
    send_action(&Action::Delete { msg_id });
}

/// Send an image via base64 data or HTTP/HTTPS URL with optional caption.
pub fn send_image(data_or_url: &str, caption: Option<&str>) {
    send_action(&Action::SendImage {
        data: data_or_url,
        caption,
        mimetype: None,
    });
}

/// Send audio/voice note via base64 data or HTTP/HTTPS URL.
pub fn send_audio(data_or_url: &str, is_ptt: bool) {
    send_action(&Action::SendAudio {
        data: data_or_url,
        mimetype: None,
        ptt: is_ptt,
    });
}

/// Send a video via base64 data or HTTP/HTTPS URL with optional caption.
pub fn send_video(data_or_url: &str, caption: Option<&str>) {
    send_action(&Action::SendVideo {
        data: data_or_url,
        caption,
        mimetype: None,
        gif_playback: false,
    });
}

/// Send a document via base64 data or HTTP/HTTPS URL with filename and optional caption.
pub fn send_document(data_or_url: &str, filename: &str, caption: Option<&str>) {
    send_action(&Action::SendDocument {
        data: data_or_url,
        filename: Some(filename),
        caption,
        mimetype: None,
    });
}

/// Send a sticker via WebP base64 data or HTTP/HTTPS URL.
pub fn send_sticker(data_or_url: &str) {
    send_action(&Action::SendSticker { data: data_or_url });
}

/// Send an interactive poll.
pub fn send_poll(question: &str, options: &[&str]) {
    send_action(&Action::Poll {
        question,
        options,
        selectable: 1,
    });
}

/// Signal completion of the plugin session.
pub fn send_done() {
    send_action(&Action::Done);
}

// ─── Simple Mode (Plain Text) ────────────────────────────────────────────────

/// Sends a plain text response to WhatsRook via stdout (simple mode).
pub fn respond(output: impl AsRef<str>) {
    print!("{}", output.as_ref().trim());
}

/// Sends an error message to stderr and stdout, then exits with code 1.
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
