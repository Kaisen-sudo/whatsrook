use serde::{Deserialize, Serialize};
use std::io::{self, IsTerminal, Read};
use std::time::Duration;

pub const DEFAULT_USER_AGENT: &str =
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36";

/// Incoming request payload sent by WhatsRook via stdin or parsed from CLI arguments.
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
}

impl Request {
    /// Loads the plugin request from stdin if piped; falls back to CLI arguments.
    pub fn load() -> Self {
        // If stdin is piped (e.g. invoked by WhatsRook or echo | binary), read it
        if !io::stdin().is_terminal() {
            let mut buffer = String::new();
            if io::stdin().read_to_string(&mut buffer).is_ok() && !buffer.trim().is_empty() {
                if let Ok(req) = serde_json::from_str::<Request>(&buffer) {
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

        // Fallback: parse from command line arguments
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
        }
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
}

/// Constructs a preconfigured blocking HTTP client with timeouts and standard headers.
pub fn create_http_client(timeout_secs: u64) -> reqwest::blocking::Client {
    reqwest::blocking::Client::builder()
        .user_agent(DEFAULT_USER_AGENT)
        .timeout(Duration::from_secs(timeout_secs))
        .build()
        .unwrap_or_default()
}

/// Sends a successful response to WhatsRook via stdout.
pub fn respond(output: impl AsRef<str>) {
    print!("{}", output.as_ref().trim());
}

/// Sends an error message to stderr and exits with failure.
pub fn respond_err(error_msg: impl AsRef<str>) -> ! {
    eprintln!("{}", error_msg.as_ref());
    // Also output user-friendly error to stdout so the chat receives feedback
    print!("{}", error_msg.as_ref().trim());
    std::process::exit(1);
}
