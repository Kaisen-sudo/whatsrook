//! # whatsrook-sdk
//!
//! Rust SDK for building [WhatsRook](https://github.com/Thruqe/whatsrook) external plugins.
//!
//! External plugins are standalone executables — written in any language — that WhatsRook
//! spawns as child processes. This crate provides everything a Rust plugin author needs:
//!
//! - **[`Request`]** — deserializes the incoming JSON payload from `stdin`.
//! - **[`Action`]** — typed, serializable frames for every response the protocol supports.
//! - **High-level helpers** — `send_reply_live`, `send_image`, `send_poll`, etc.
//! - **[`create_http_client`]** — preconfigured blocking HTTP client.
//!
//! ## Communication Protocol
//!
//! WhatsRook writes one JSON line to the plugin's `stdin` when a command is triggered.
//! The plugin reads it, does its work, and writes newline-delimited JSON *action frames*
//! to `stdout`. WhatsRook executes each frame and, for actions that return a message ID,
//! writes an [`Ack`] back to the plugin's `stdin`.
//!
//! ```text
//! WhatsRook  ──stdin──▶  plugin (reads Request)
//! plugin     ──stdout─▶  WhatsRook (reads Action frames)
//! WhatsRook  ──stdin──▶  plugin (reads Ack, for live actions)
//! ```
//!
//! ## Quick Start
//!
//! ```no_run
//! use whatsrook_sdk::{respond, Request};
//!
//! fn main() {
//!     let req = Request::load();
//!     let query = req.query();
//!
//!     if query.is_empty() {
//!         respond("Usage: .hello <name>");
//!         return;
//!     }
//!
//!     respond(format!("Hello, {}! 👋", query));
//! }
//! ```
//!
//! See the [`examples/`](https://github.com/Thruqe/whatsrook-sdk/tree/master/examples) directory
//! for more complete plugin examples.

use serde::{Deserialize, Serialize};
use std::io::{self, BufRead, IsTerminal, Write};
use std::time::Duration;

pub use reqwest;
pub use reqwest::blocking::Client as HttpClient;

pub mod fmt;
pub mod media;

pub use fmt::{
    MessageBuilder, bold, bullet_list, code_block, italic, monospace, numbered_list, quote,
    strikethrough,
};
pub use media::{
    Base64Error, decode_base64, encode_base64, read_file_as_base64, read_file_as_data_url,
    to_data_url,
};

/// Default browser User-Agent used by [`create_http_client`].
pub const DEFAULT_USER_AGENT: &str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36";

// ─── Request ─────────────────────────────────────────────────────────────────

/// Quoted message context payload — the message the user was replying to.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct QuotedMessage {
    /// WhatsApp message ID of the quoted message.
    #[serde(default)]
    pub id: String,
    /// JID of the sender of the quoted message.
    #[serde(default)]
    pub sender: String,
    /// Text content of the quoted message.
    #[serde(default)]
    pub text: String,
}

/// Incoming request payload written by WhatsRook to the plugin's `stdin`.
///
/// Load it at the start of `main` with [`Request::load`]:
///
/// ```no_run
/// use whatsrook_sdk::Request;
///
/// fn main() {
///     let req = Request::load();
///     println!("command: {}", req.command);
///     println!("args: {:?}", req.args);
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Request {
    /// The installed plugin command name (e.g. `"weather"`).
    #[serde(default)]
    pub command: String,
    /// Whitespace-split arguments following the command.
    #[serde(default)]
    pub args: Vec<String>,
    /// Full unparsed argument string following the command.
    #[serde(default)]
    pub raw_args: String,
    /// JID of the WhatsApp chat where the command was triggered.
    #[serde(default)]
    pub chat: String,
    /// JID of the user who triggered the command.
    #[serde(default)]
    pub sender: String,
    /// The bot's active command prefix (e.g. `"."` or `"/"`).
    #[serde(default)]
    pub prefix: String,
    /// The configured bot display name.
    #[serde(default)]
    pub bot_name: String,
    /// The WhatsApp push name (display name) of the sender.
    #[serde(default)]
    pub push_name: String,
    /// `true` if the command was triggered inside a group chat.
    #[serde(default)]
    pub is_group: bool,
    /// `true` if the sender is in the bot's sudoers list or is the owner.
    #[serde(default)]
    pub is_sudo: bool,
    /// `true` if the sender is the primary bot owner.
    #[serde(default)]
    pub is_owner: bool,
    /// `true` if the sender is a group admin in the triggering chat.
    #[serde(default)]
    pub is_admin: bool,
    /// `true` if a live session is already active for this plugin in this chat.
    #[serde(default)]
    pub live_session: bool,
    /// `true` if the user is requesting cancellation of an active live session.
    #[serde(default)]
    pub is_cancel_request: bool,
    /// Quoted message context, present only when the triggering message is a reply.
    #[serde(default)]
    pub quoted_message: Option<QuotedMessage>,
    /// JIDs of users mentioned in the triggering message.
    #[serde(default)]
    pub mentioned_jids: Vec<String>,
}

impl Request {
    /// Loads the plugin request from `stdin` (first line as JSON).
    ///
    /// When `stdin` is not a terminal (i.e. the plugin is invoked by WhatsRook),
    /// the first line is read and parsed as the JSON payload. If reading or
    /// parsing fails, or if `stdin` is a terminal (direct invocation), the
    /// function falls back to building a minimal [`Request`] from CLI arguments,
    /// which is useful for local development and testing.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use whatsrook_sdk::Request;
    ///
    /// fn main() {
    ///     let req = Request::load();
    ///     println!("query: {}", req.query());
    /// }
    /// ```
    pub fn load() -> Self {
        if !io::stdin().is_terminal() {
            let stdin = io::stdin();
            let mut line = String::new();
            if stdin.lock().read_line(&mut line).is_ok()
                && !line.trim().is_empty()
                && let Ok(req) = serde_json::from_str::<Request>(line.trim())
                && (!req.command.is_empty()
                    || !req.args.is_empty()
                    || !req.raw_args.is_empty()
                    || !req.chat.is_empty())
            {
                return req;
            }
        }

        // Fallback: build a minimal request from CLI args for local testing.
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

    /// Alias for [`load`](Request::load) — loads the request from `stdin`.
    ///
    /// Useful in streaming / live-session plugin contexts to make the intent explicit.
    pub fn load_streaming() -> Self {
        Self::load()
    }

    /// Returns the trimmed `raw_args` string, falling back to joining `args` with spaces.
    ///
    /// This is the primary way to get the user's input to a plugin:
    ///
    /// ```no_run
    /// use whatsrook_sdk::Request;
    ///
    /// fn main() {
    ///     let req = Request::load();
    ///     let q = req.query();
    ///     if q.is_empty() {
    ///         whatsrook_sdk::respond("Usage: .myplugin <query>");
    ///         return;
    ///     }
    ///     whatsrook_sdk::respond(format!("You said: {}", q));
    /// }
    /// ```
    pub fn query(&self) -> String {
        let trimmed = self.raw_args.trim();
        if !trimmed.is_empty() {
            trimmed.to_string()
        } else {
            self.args.join(" ").trim().to_string()
        }
    }

    /// Returns the effective command prefix. Falls back to `"."` if the field is empty.
    pub fn prefix(&self) -> &str {
        if self.prefix.is_empty() {
            "."
        } else {
            &self.prefix
        }
    }

    /// Returns the bot display name. Falls back to `"WhatsRook"` if the field is empty.
    pub fn bot_name(&self) -> &str {
        if self.bot_name.is_empty() {
            "WhatsRook"
        } else {
            &self.bot_name
        }
    }

    /// Returns the sender's WhatsApp push name. Falls back to `"User"` if empty.
    pub fn push_name(&self) -> &str {
        if self.push_name.is_empty() {
            "User"
        } else {
            &self.push_name
        }
    }

    /// Returns the text of the quoted message, if present.
    pub fn quoted_text(&self) -> Option<&str> {
        self.quoted_message.as_ref().map(|q| q.text.as_str())
    }

    /// Returns the JID of the sender of the quoted message, if present.
    pub fn quoted_sender(&self) -> Option<&str> {
        self.quoted_message.as_ref().map(|q| q.sender.as_str())
    }

    /// Returns the message ID of the quoted message, if present.
    pub fn quoted_id(&self) -> Option<&str> {
        self.quoted_message.as_ref().map(|q| q.id.as_str())
    }

    /// Returns `true` if the command was triggered inside a group chat.
    pub fn is_group(&self) -> bool {
        self.is_group
    }

    /// Returns `true` if the sender is a sudo user or the bot owner.
    pub fn is_sudo(&self) -> bool {
        self.is_sudo
    }

    /// Returns `true` if the sender is the primary bot owner.
    pub fn is_owner(&self) -> bool {
        self.is_owner
    }

    /// Returns `true` if the sender is a group admin in the triggering chat.
    pub fn is_admin(&self) -> bool {
        self.is_admin
    }

    /// Returns the argument at 0-based `index`, if available.
    pub fn arg(&self, index: usize) -> Option<&str> {
        self.args.get(index).map(|s| s.as_str())
    }

    /// Parses the argument at 0-based `index` into any type implementing [`std::str::FromStr`].
    pub fn arg_as<T: std::str::FromStr>(&self, index: usize) -> Option<T> {
        self.arg(index).and_then(|s| s.parse::<T>().ok())
    }

    /// Returns the first argument (subcommand) if present.
    pub fn subcommand(&self) -> Option<&str> {
        self.arg(0)
    }

    /// Returns all arguments following the first argument (subcommand).
    pub fn subcommand_args(&self) -> &[String] {
        if self.args.is_empty() {
            &[]
        } else {
            &self.args[1..]
        }
    }

    /// Checks if a boolean flag (e.g. `--help` or `-h`) was passed in the arguments.
    pub fn flag(&self, name: &str) -> bool {
        let long = format!("--{}", name);
        let short = format!("-{}", name);
        self.args.iter().any(|a| a == &long || a == &short)
    }

    /// Returns the string value of a flag passed as `--<name>=<val>` or `--<name> <val>`.
    pub fn flag_value(&self, name: &str) -> Option<&str> {
        let prefix = format!("--{}=", name);
        let long = format!("--{}", name);
        let short = format!("-{}", name);
        for (i, a) in self.args.iter().enumerate() {
            if let Some(val) = a.strip_prefix(&prefix) {
                return Some(val);
            }
            if (a == &long || a == &short) && i + 1 < self.args.len() {
                return Some(self.args[i + 1].as_str());
            }
        }
        None
    }

    /// Returns `true` if the message was sent in a direct message (private chat), not in a group.
    pub fn is_dm(&self) -> bool {
        !self.is_group
    }

    /// Returns `true` if a live session is currently active.
    pub fn is_live(&self) -> bool {
        self.live_session
    }

    /// Returns `true` if this invocation is requesting cancellation of an active live session.
    pub fn is_cancel(&self) -> bool {
        self.is_cancel_request
    }

    /// Returns the target chat JID.
    pub fn chat_id(&self) -> &str {
        &self.chat
    }

    /// Returns the sender's JID.
    pub fn sender_id(&self) -> &str {
        &self.sender
    }

    /// Extracts the phone number / user ID from the sender JID (part before `@`).
    pub fn sender_phone(&self) -> Option<&str> {
        if self.sender.is_empty() {
            None
        } else {
            self.sender.split('@').next()
        }
    }

    /// Extracts the target ID from the chat JID (part before `@`).
    pub fn chat_target(&self) -> Option<&str> {
        if self.chat.is_empty() {
            None
        } else {
            self.chat.split('@').next()
        }
    }

    /// Returns the slice of mentioned user JIDs.
    pub fn mentioned_jids(&self) -> &[String] {
        &self.mentioned_jids
    }
}

// ─── Action Protocol ──────────────────────────────────────────────────────────

/// A single action frame written to `stdout` for WhatsRook to execute.
///
/// Use [`send_action`] to serialize and flush a frame, or use the high-level
/// helpers (`send_reply_live`, `send_image`, `send_poll`, etc.) which call it
/// internally.
///
/// # Wire Format
///
/// Each variant is serialized as a JSON object with an `"action"` field that
/// identifies the type (snake_case), plus any variant-specific fields:
///
/// ```json
/// { "action": "reply", "text": "Hello world" }
/// { "action": "send_image", "data": "https://example.com/img.png", "caption": "Look!" }
/// { "action": "done" }
/// ```
#[derive(Debug, Serialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum Action<'a> {
    /// Send a text reply.
    ///
    /// WhatsRook sends back an [`Ack`] on `stdin` with the `msg_id` of the sent
    /// message, which can be used for subsequent [`Edit`](Action::Edit) frames.
    Reply { text: &'a str },

    /// Edit an existing message by its ID, replacing the text in-place.
    ///
    /// Requires the `msg_id` returned by a prior [`Reply`](Action::Reply) ack.
    Edit { msg_id: &'a str, text: &'a str },

    /// React to the triggering message (or a specific message) with an emoji.
    ///
    /// Omitting `msg_id` reacts to the command message that triggered the plugin.
    React {
        /// Target message ID. `None` reacts to the triggering message.
        #[serde(skip_serializing_if = "Option::is_none")]
        msg_id: Option<&'a str>,
        /// Emoji character to react with (e.g. `"🔥"`).
        emoji: &'a str,
    },

    /// Revoke (delete for everyone) a message by its ID.
    Delete { msg_id: &'a str },

    /// Send an image from a base64-encoded string or an HTTP/HTTPS URL.
    SendImage {
        /// Image data — base64 string or a URL.
        data: &'a str,
        /// Optional caption displayed below the image.
        #[serde(skip_serializing_if = "Option::is_none")]
        caption: Option<&'a str>,
        /// Optional MIME type override (e.g. `"image/webp"`).
        #[serde(skip_serializing_if = "Option::is_none")]
        mimetype: Option<&'a str>,
    },

    /// Send audio or a voice note (PTT) from a base64-encoded string or a URL.
    SendAudio {
        /// Audio data — base64 string or a URL.
        data: &'a str,
        /// Optional MIME type override (e.g. `"audio/ogg; codecs=opus"`).
        #[serde(skip_serializing_if = "Option::is_none")]
        mimetype: Option<&'a str>,
        /// `true` to send as a push-to-talk voice note.
        #[serde(default)]
        ptt: bool,
    },

    /// Send a video or looping GIF from a base64-encoded string or a URL.
    SendVideo {
        /// Video data — base64 string or a URL.
        data: &'a str,
        /// Optional caption displayed below the video.
        #[serde(skip_serializing_if = "Option::is_none")]
        caption: Option<&'a str>,
        /// Optional MIME type override.
        #[serde(skip_serializing_if = "Option::is_none")]
        mimetype: Option<&'a str>,
        /// `true` to render the video as a looping GIF.
        #[serde(default)]
        gif_playback: bool,
    },

    /// Send a document file from a base64-encoded string or a URL.
    SendDocument {
        /// Document data — base64 string or a URL.
        data: &'a str,
        /// Filename shown in the WhatsApp document message (e.g. `"report.pdf"`).
        #[serde(skip_serializing_if = "Option::is_none")]
        filename: Option<&'a str>,
        /// Optional caption displayed below the document.
        #[serde(skip_serializing_if = "Option::is_none")]
        caption: Option<&'a str>,
        /// Optional MIME type override.
        #[serde(skip_serializing_if = "Option::is_none")]
        mimetype: Option<&'a str>,
    },

    /// Send a WebP sticker from a base64-encoded string or a URL.
    SendSticker {
        /// Sticker data — base64 string or a URL to a `.webp` file.
        data: &'a str,
    },

    /// Send an interactive poll.
    Poll {
        /// The poll question shown to participants.
        question: &'a str,
        /// The list of answer options. Maximum of 12 options.
        options: &'a [&'a str],
        /// Number of options a participant may select (`1` = single-select).
        #[serde(default)]
        selectable: usize,
    },

    /// Show or update the typing / processing loader indicator.
    ///
    /// Pass `None` to show a generic loader, or `Some(text)` for a status message.
    Loader {
        /// Optional status text shown alongside the indicator.
        #[serde(skip_serializing_if = "Option::is_none")]
        text: Option<&'a str>,
    },

    /// Signal that the plugin has finished all work and the session is complete.
    ///
    /// Always send this as the last frame of a live session.
    Done,
}

impl<'a> Action<'a> {
    /// Creates a text reply action frame.
    pub fn reply(text: &'a str) -> Self {
        Action::Reply { text }
    }

    /// Creates an edit action frame targeting an existing message ID.
    pub fn edit(msg_id: &'a str, text: &'a str) -> Self {
        Action::Edit { msg_id, text }
    }

    /// Creates a reaction action frame on the triggering message.
    pub fn react(emoji: &'a str) -> Self {
        Action::React {
            msg_id: None,
            emoji,
        }
    }

    /// Creates a reaction action frame on a specific message by its ID.
    pub fn react_to(msg_id: &'a str, emoji: &'a str) -> Self {
        Action::React {
            msg_id: Some(msg_id),
            emoji,
        }
    }

    /// Creates a delete action frame to revoke a message for everyone.
    pub fn delete(msg_id: &'a str) -> Self {
        Action::Delete { msg_id }
    }

    /// Creates an image action frame.
    pub fn image(data: &'a str) -> Self {
        Action::SendImage {
            data,
            caption: None,
            mimetype: None,
        }
    }

    /// Creates an audio action frame.
    pub fn audio(data: &'a str) -> Self {
        Action::SendAudio {
            data,
            mimetype: None,
            ptt: false,
        }
    }

    /// Creates a video action frame.
    pub fn video(data: &'a str) -> Self {
        Action::SendVideo {
            data,
            caption: None,
            mimetype: None,
            gif_playback: false,
        }
    }

    /// Creates a document action frame.
    pub fn document(data: &'a str, filename: &'a str) -> Self {
        Action::SendDocument {
            data,
            filename: Some(filename),
            caption: None,
            mimetype: None,
        }
    }

    /// Creates a sticker action frame.
    pub fn sticker(data: &'a str) -> Self {
        Action::SendSticker { data }
    }

    /// Creates a single-select poll action frame.
    pub fn poll(question: &'a str, options: &'a [&'a str]) -> Self {
        Action::Poll {
            question,
            options,
            selectable: 1,
        }
    }

    /// Creates a loader action frame.
    pub fn loader(text: Option<&'a str>) -> Self {
        Action::Loader { text }
    }

    /// Creates a done action frame signaling session completion.
    pub fn done() -> Self {
        Action::Done
    }

    /// Sets or updates the caption on an image, video, or document action.
    pub fn with_caption(mut self, cap: &'a str) -> Self {
        match &mut self {
            Action::SendImage { caption, .. } => *caption = Some(cap),
            Action::SendVideo { caption, .. } => *caption = Some(cap),
            Action::SendDocument { caption, .. } => *caption = Some(cap),
            _ => {}
        }
        self
    }

    /// Sets or updates the MIME type override.
    pub fn with_mimetype(mut self, mime: &'a str) -> Self {
        match &mut self {
            Action::SendImage { mimetype, .. } => *mimetype = Some(mime),
            Action::SendAudio { mimetype, .. } => *mimetype = Some(mime),
            Action::SendVideo { mimetype, .. } => *mimetype = Some(mime),
            Action::SendDocument { mimetype, .. } => *mimetype = Some(mime),
            _ => {}
        }
        self
    }

    /// Configures audio playback as a push-to-talk voice note (PTT).
    pub fn as_ptt(mut self, is_ptt: bool) -> Self {
        if let Action::SendAudio { ptt, .. } = &mut self {
            *ptt = is_ptt;
        }
        self
    }

    /// Configures video playback as a looping GIF.
    pub fn as_gif(mut self, is_gif: bool) -> Self {
        if let Action::SendVideo { gif_playback, .. } = &mut self {
            *gif_playback = is_gif;
        }
        self
    }

    /// Configures the number of selectable options for a poll.
    pub fn with_selectable(mut self, count: usize) -> Self {
        if let Action::Poll { selectable, .. } = &mut self {
            *selectable = count;
        }
        self
    }

    /// Serializes and sends this action to `stdout` immediately.
    pub fn send(&self) {
        send_action(self);
    }
}

/// Acknowledgement sent by WhatsRook on `stdin` following actions that return a message ID.
///
/// Currently returned after [`Action::Reply`]. Read it with [`await_ack`].
#[derive(Debug, Deserialize)]
pub struct Ack {
    /// `true` if the action succeeded.
    pub ok: bool,
    /// The message ID of the sent message. Present when `ok` is `true`.
    pub msg_id: Option<String>,
    /// Human-readable error description. Present when `ok` is `false`.
    pub error: Option<String>,
}

// ─── Low-Level I/O ───────────────────────────────────────────────────────────

/// Serialize an [`Action`] frame to `stdout` and flush immediately.
///
/// Each frame is a single JSON line followed by a newline character. WhatsRook
/// reads frames as they arrive, so flushing after each write is essential for
/// correct live-session behavior.
///
/// Prefer the high-level helpers (`send_reply_live`, `send_image`, etc.) over
/// calling this function directly unless you need fine-grained control.
pub fn send_action(action: &Action) {
    let json = serde_json::to_string(action).unwrap_or_default();
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    let _ = writeln!(handle, "{}", json);
    let _ = handle.flush();
}

/// Read one [`Ack`] line from `stdin`. Returns `None` on EOF or parse error.
///
/// Call this immediately after [`send_reply_live`] (which calls it internally)
/// when you need the raw `Ack` for error inspection. In most cases the
/// high-level helpers handle the round-trip for you.
pub fn await_ack() -> Option<Ack> {
    let stdin = io::stdin();
    let mut line = String::new();
    stdin.lock().read_line(&mut line).ok()?;
    serde_json::from_str(line.trim()).ok()
}

// ─── High-Level Action Helpers ────────────────────────────────────────────────

/// Send a text `reply` action, await the acknowledgement, and return the
/// `msg_id` on success.
///
/// The returned `msg_id` is required for subsequent in-place edits via
/// [`send_edit_live`]. Returns `None` if the reply failed or the ACK could
/// not be read.
///
/// # Example
///
/// ```no_run
/// use whatsrook_sdk::{send_edit_live, send_reply_live};
///
/// fn main() {
///     if let Some(msg_id) = send_reply_live("⏳ Processing...") {
///         // ... do work ...
///         send_edit_live(&msg_id, "✅ Done!");
///     }
/// }
/// ```
pub fn send_reply_live(text: &str) -> Option<String> {
    send_action(&Action::Reply { text });
    let ack = await_ack()?;
    if ack.ok { ack.msg_id } else { None }
}

/// Send an `edit` frame to update a previously sent message in-place.
///
/// `msg_id` must be a value returned by [`send_reply_live`].
pub fn send_edit_live(msg_id: &str, text: &str) {
    send_action(&Action::Edit { msg_id, text });
}

/// React to the triggering message with an emoji.
///
/// # Example
///
/// ```no_run
/// whatsrook_sdk::send_react("🚀");
/// ```
pub fn send_react(emoji: &str) {
    send_action(&Action::React {
        msg_id: None,
        emoji,
    });
}

/// React to a specific message ID with an emoji.
///
/// # Example
///
/// ```no_run
/// whatsrook_sdk::send_react_to("3EB01234567890ABCDEF", "🔥");
/// ```
pub fn send_react_to(msg_id: &str, emoji: &str) {
    send_action(&Action::React {
        msg_id: Some(msg_id),
        emoji,
    });
}

/// Revoke (delete for everyone) a message by its ID.
pub fn send_delete(msg_id: &str) {
    send_action(&Action::Delete { msg_id });
}

/// Send an image from a URL or base64-encoded data with an optional caption.
///
/// # Example
///
/// ```no_run
/// whatsrook_sdk::send_image("https://example.com/chart.png", Some("📊 Market chart"));
/// ```
pub fn send_image(data_or_url: &str, caption: Option<&str>) {
    send_action(&Action::SendImage {
        data: data_or_url,
        caption,
        mimetype: None,
    });
}

/// Send an image with a custom MIME type and optional caption.
pub fn send_image_with_type(data_or_url: &str, caption: Option<&str>, mimetype: Option<&str>) {
    send_action(&Action::SendImage {
        data: data_or_url,
        caption,
        mimetype,
    });
}

/// Send audio or a voice note (PTT) from a URL or base64-encoded data.
///
/// Set `is_ptt` to `true` to render the message as a push-to-talk voice note
/// instead of a standard audio attachment.
pub fn send_audio(data_or_url: &str, is_ptt: bool) {
    send_action(&Action::SendAudio {
        data: data_or_url,
        mimetype: None,
        ptt: is_ptt,
    });
}

/// Send audio with an explicit MIME type and PTT flag.
pub fn send_audio_full(data_or_url: &str, is_ptt: bool, mimetype: Option<&str>) {
    send_action(&Action::SendAudio {
        data: data_or_url,
        mimetype,
        ptt: is_ptt,
    });
}

/// Send a video from a URL or base64-encoded data with an optional caption.
pub fn send_video(data_or_url: &str, caption: Option<&str>) {
    send_action(&Action::SendVideo {
        data: data_or_url,
        caption,
        mimetype: None,
        gif_playback: false,
    });
}

/// Send a GIF (looping video) from a URL or base64-encoded data.
pub fn send_gif(data_or_url: &str, caption: Option<&str>) {
    send_action(&Action::SendVideo {
        data: data_or_url,
        caption,
        mimetype: None,
        gif_playback: true,
    });
}

/// Send a video with full options: caption, MIME type override, and GIF playback mode.
pub fn send_video_full(
    data_or_url: &str,
    caption: Option<&str>,
    mimetype: Option<&str>,
    gif_playback: bool,
) {
    send_action(&Action::SendVideo {
        data: data_or_url,
        caption,
        mimetype,
        gif_playback,
    });
}

/// Send a document file from a URL or base64-encoded data.
///
/// `filename` is the name shown in the WhatsApp document bubble (e.g. `"report.pdf"`).
pub fn send_document(data_or_url: &str, filename: &str, caption: Option<&str>) {
    send_action(&Action::SendDocument {
        data: data_or_url,
        filename: Some(filename),
        caption,
        mimetype: None,
    });
}

/// Send a document file with full options including MIME type override.
pub fn send_document_full(
    data_or_url: &str,
    filename: &str,
    caption: Option<&str>,
    mimetype: Option<&str>,
) {
    send_action(&Action::SendDocument {
        data: data_or_url,
        filename: Some(filename),
        caption,
        mimetype,
    });
}

/// Send a WebP sticker from a URL or base64-encoded data.
pub fn send_sticker(data_or_url: &str) {
    send_action(&Action::SendSticker { data: data_or_url });
}

/// Send an interactive single-select poll.
///
/// # Example
///
/// ```no_run
/// whatsrook_sdk::send_poll("Which network?", &["Bitcoin", "Ethereum", "Solana"]);
/// ```
pub fn send_poll(question: &str, options: &[&str]) {
    send_action(&Action::Poll {
        question,
        options,
        selectable: 1,
    });
}

/// Send an interactive multi-choice poll with a customizable number of selectable options.
///
/// # Example
///
/// ```no_run
/// whatsrook_sdk::send_multi_poll("Pick up to 2:", &["Rust", "Go", "TypeScript"], 2);
/// ```
pub fn send_multi_poll(question: &str, options: &[&str], selectable: usize) {
    send_action(&Action::Poll {
        question,
        options,
        selectable,
    });
}

/// Show the typing / processing loader indicator with an optional status message.
///
/// Call this before starting a long-running operation to give the user
/// immediate feedback that the plugin is working.
///
/// # Example
///
/// ```no_run
/// whatsrook_sdk::send_loader(Some("Fetching data..."));
/// // ... long operation ...
/// whatsrook_sdk::send_done();
/// ```
pub fn send_loader(text: Option<&str>) {
    send_action(&Action::Loader { text });
}

/// Signal that the plugin session is complete.
///
/// Always send this as the final frame in a live session so WhatsRook can
/// clean up session state for the chat.
pub fn send_done() {
    send_action(&Action::Done);
}

// ─── Simple Mode ─────────────────────────────────────────────────────────────

/// Send a plain-text response to WhatsRook via `stdout` (simple mode).
///
/// Simple mode is the easiest output method: write a single string to `stdout`
/// and WhatsRook sends it as a text message. No JSON framing is required.
///
/// Do **not** mix simple-mode output (`respond`) with action-frame output
/// (`send_action` / `send_reply_live`) in the same plugin run.
///
/// # Example
///
/// ```no_run
/// use whatsrook_sdk::{respond, Request};
///
/// fn main() {
///     let req = Request::load();
///     respond(format!("Hello, {}!", req.push_name()));
/// }
/// ```
pub fn respond(output: impl AsRef<str>) {
    print!("{}", output.as_ref().trim());
}

/// Send an error message to `stderr` (for logs) and `stdout` (to reply to the user),
/// then exit the process with code `1`.
///
/// # Panics / Exits
///
/// This function **never returns** — it always calls [`std::process::exit`] with code `1`.
pub fn respond_err(error_msg: impl AsRef<str>) -> ! {
    eprintln!("{}", error_msg.as_ref());
    print!("{}", error_msg.as_ref().trim());
    std::process::exit(1)
}

// ─── HTTP Client ──────────────────────────────────────────────────────────────

/// Build a preconfigured blocking [`HttpClient`] with a timeout and standard browser headers.
///
/// The client uses `rustls` for TLS and sets a browser-like `User-Agent` by default.
/// Most external plugins that make HTTP requests should use this instead of
/// constructing their own client.
///
/// # Arguments
///
/// * `timeout_secs` — maximum time in seconds for a single HTTP request to complete.
///
/// # Example
///
/// ```no_run
/// let client = whatsrook_sdk::create_http_client(10);
/// let body = client.get("https://api.example.com/data").send()?.text()?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn create_http_client(timeout_secs: u64) -> reqwest::blocking::Client {
    reqwest::blocking::Client::builder()
        .user_agent(DEFAULT_USER_AGENT)
        .timeout(Duration::from_secs(timeout_secs))
        .build()
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_methods() {
        let req = Request {
            command: "weather".into(),
            args: vec![
                "forecast".into(),
                "London".into(),
                "--days".into(),
                "5".into(),
                "-v".into(),
            ],
            raw_args: "forecast London --days 5 -v".into(),
            chat: "123456789-987654@g.us".into(),
            sender: "44712345678@s.whatsapp.net".into(),
            prefix: ".".into(),
            bot_name: "WhatsRook".into(),
            push_name: "Alice".into(),
            is_group: true,
            is_sudo: false,
            is_owner: false,
            is_admin: true,
            live_session: true,
            is_cancel_request: false,
            quoted_message: None,
            mentioned_jids: vec!["44799999999@s.whatsapp.net".into()],
        };

        assert_eq!(req.subcommand(), Some("forecast"));
        assert_eq!(req.subcommand_args(), &["London", "--days", "5", "-v"]);
        assert_eq!(req.arg(1), Some("London"));
        assert_eq!(req.arg_as::<u32>(3), Some(5));
        assert!(req.flag("v"));
        assert_eq!(req.flag_value("days"), Some("5"));
        assert_eq!(req.sender_phone(), Some("44712345678"));
        assert_eq!(req.chat_target(), Some("123456789-987654"));
        assert_eq!(req.chat_id(), "123456789-987654@g.us");
        assert_eq!(req.sender_id(), "44712345678@s.whatsapp.net");
        assert!(!req.is_dm());
        assert!(req.is_group());
        assert!(req.is_admin());
        assert!(req.is_live());
        assert!(!req.is_cancel());
        assert_eq!(req.mentioned_jids().len(), 1);
    }

    #[test]
    fn test_action_builders_and_serialization() {
        let reply = Action::reply("test message");
        let json = serde_json::to_string(&reply).unwrap();
        assert_eq!(json, r#"{"action":"reply","text":"test message"}"#);

        let edit = Action::edit("MSG123", "updated");
        let json = serde_json::to_string(&edit).unwrap();
        assert_eq!(
            json,
            r#"{"action":"edit","msg_id":"MSG123","text":"updated"}"#
        );

        let react = Action::react("👍");
        let json = serde_json::to_string(&react).unwrap();
        assert_eq!(json, r#"{"action":"react","emoji":"👍"}"#);

        let react_to = Action::react_to("MSG456", "🔥");
        let json = serde_json::to_string(&react_to).unwrap();
        assert_eq!(json, r#"{"action":"react","msg_id":"MSG456","emoji":"🔥"}"#);

        let image = Action::image("https://example.com/pic.png").with_caption("A photo");
        let json = serde_json::to_string(&image).unwrap();
        assert_eq!(
            json,
            r#"{"action":"send_image","data":"https://example.com/pic.png","caption":"A photo"}"#
        );

        let video = Action::video("https://example.com/vid.mp4").as_gif(true);
        let json = serde_json::to_string(&video).unwrap();
        assert_eq!(
            json,
            r#"{"action":"send_video","data":"https://example.com/vid.mp4","gif_playback":true}"#
        );

        let poll = Action::poll("Choose:", &["A", "B", "C"]).with_selectable(2);
        let json = serde_json::to_string(&poll).unwrap();
        assert_eq!(
            json,
            r#"{"action":"poll","question":"Choose:","options":["A","B","C"],"selectable":2}"#
        );
    }
}
