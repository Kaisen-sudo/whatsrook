//! WhatsApp text formatting helpers and fluent message builder.

/// Formats text in *bold* for WhatsApp (`*text*`).
///
/// # Example
///
/// ```
/// use whatsrook_sdk::fmt::bold;
/// assert_eq!(bold("hello"), "*hello*");
/// ```
pub fn bold(text: impl AsRef<str>) -> String {
    format!("*{}*", text.as_ref())
}

/// Formats text in _italics_ for WhatsApp (`_text_`).
///
/// # Example
///
/// ```
/// use whatsrook_sdk::fmt::italic;
/// assert_eq!(italic("hello"), "_hello_");
/// ```
pub fn italic(text: impl AsRef<str>) -> String {
    format!("_{}_", text.as_ref())
}

/// Formats text with ~strikethrough~ for WhatsApp (`~text~`).
///
/// # Example
///
/// ```
/// use whatsrook_sdk::fmt::strikethrough;
/// assert_eq!(strikethrough("hello"), "~hello~");
/// ```
pub fn strikethrough(text: impl AsRef<str>) -> String {
    format!("~{}~", text.as_ref())
}

/// Formats text in `monospace` (inline code) for WhatsApp (`` `text` ``).
///
/// # Example
///
/// ```
/// use whatsrook_sdk::fmt::monospace;
/// assert_eq!(monospace("cargo test"), "`cargo test`");
/// ```
pub fn monospace(text: impl AsRef<str>) -> String {
    format!("`{}`", text.as_ref())
}

/// Formats text in a WhatsApp multi-line code block (```` ```code``` ````).
///
/// # Example
///
/// ```
/// use whatsrook_sdk::fmt::code_block;
/// assert_eq!(code_block("println!(\"hi\");"), "```\nprintln!(\"hi\");\n```");
/// ```
pub fn code_block(text: impl AsRef<str>) -> String {
    format!("```\n{}\n```", text.as_ref())
}

/// Formats text as a blockquote for WhatsApp (`> text`).
///
/// Each line of multi-line input is prefixed with `> `.
///
/// # Example
///
/// ```
/// use whatsrook_sdk::fmt::quote;
/// assert_eq!(quote("line 1\nline 2"), "> line 1\n> line 2");
/// ```
pub fn quote(text: impl AsRef<str>) -> String {
    text.as_ref()
        .lines()
        .map(|line| format!("> {}", line))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Formats an iterator of items as a bulleted list (`• item`).
///
/// # Example
///
/// ```
/// use whatsrook_sdk::fmt::bullet_list;
/// let items = vec!["First", "Second"];
/// assert_eq!(bullet_list(&items), "• First\n• Second");
/// ```
pub fn bullet_list<I, S>(items: I) -> String
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    items
        .into_iter()
        .map(|item| format!("• {}", item.as_ref()))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Formats an iterator of items as a 1-indexed numbered list (`1. item`).
///
/// # Example
///
/// ```
/// use whatsrook_sdk::fmt::numbered_list;
/// let items = vec!["Apple", "Banana"];
/// assert_eq!(numbered_list(&items), "1. Apple\n2. Banana");
/// ```
pub fn numbered_list<I, S>(items: I) -> String
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    items
        .into_iter()
        .enumerate()
        .map(|(i, item)| format!("{}. {}", i + 1, item.as_ref()))
        .collect::<Vec<_>>()
        .join("\n")
}

/// A fluent builder for constructing rich, formatted WhatsApp messages.
///
/// # Example
///
/// ```
/// use whatsrook_sdk::MessageBuilder;
///
/// let msg = MessageBuilder::new()
///     .header("System Status")
///     .bullet("CPU: Normal")
///     .bullet("Memory: 42%")
///     .newline()
///     .quote("All systems operational.")
///     .build();
///
/// assert!(msg.contains("*System Status*"));
/// assert!(msg.contains("• CPU: Normal"));
/// assert!(msg.contains("> All systems operational."));
/// ```
#[derive(Debug, Clone, Default)]
pub struct MessageBuilder {
    buffer: String,
}

impl MessageBuilder {
    /// Creates a new empty `MessageBuilder`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Appends plain text without a trailing newline.
    pub fn text(mut self, text: impl AsRef<str>) -> Self {
        self.buffer.push_str(text.as_ref());
        self
    }

    /// Appends text followed by a newline.
    pub fn line(mut self, text: impl AsRef<str>) -> Self {
        self.buffer.push_str(text.as_ref());
        self.buffer.push('\n');
        self
    }

    /// Appends a blank newline.
    pub fn newline(mut self) -> Self {
        self.buffer.push('\n');
        self
    }

    /// Appends text wrapped in *bold* (`*text*`).
    pub fn bold(mut self, text: impl AsRef<str>) -> Self {
        self.buffer.push('*');
        self.buffer.push_str(text.as_ref());
        self.buffer.push('*');
        self
    }

    /// Appends text wrapped in _italics_ (`_text_`).
    pub fn italic(mut self, text: impl AsRef<str>) -> Self {
        self.buffer.push('_');
        self.buffer.push_str(text.as_ref());
        self.buffer.push('_');
        self
    }

    /// Appends text wrapped in ~strikethrough~ (`~text~`).
    pub fn strike(mut self, text: impl AsRef<str>) -> Self {
        self.buffer.push('~');
        self.buffer.push_str(text.as_ref());
        self.buffer.push('~');
        self
    }

    /// Appends text in `monospace` (`` `text` ``).
    pub fn mono(mut self, text: impl AsRef<str>) -> Self {
        self.buffer.push('`');
        self.buffer.push_str(text.as_ref());
        self.buffer.push('`');
        self
    }

    /// Appends a multi-line code block.
    pub fn code_block(mut self, code: impl AsRef<str>) -> Self {
        self.buffer.push_str("```\n");
        self.buffer.push_str(code.as_ref());
        self.buffer.push_str("\n```\n");
        self
    }

    /// Appends each line of text quoted with `> `.
    pub fn quote(mut self, text: impl AsRef<str>) -> Self {
        for line in text.as_ref().lines() {
            self.buffer.push_str("> ");
            self.buffer.push_str(line);
            self.buffer.push('\n');
        }
        self
    }

    /// Appends a bulleted item (`• text\n`).
    pub fn bullet(mut self, text: impl AsRef<str>) -> Self {
        self.buffer.push_str("• ");
        self.buffer.push_str(text.as_ref());
        self.buffer.push('\n');
        self
    }

    /// Appends a numbered item (`{index}. text\n`).
    pub fn numbered(mut self, index: usize, text: impl AsRef<str>) -> Self {
        self.buffer.push_str(&format!("{}. ", index));
        self.buffer.push_str(text.as_ref());
        self.buffer.push('\n');
        self
    }

    /// Appends a bold header line followed by a blank line.
    pub fn header(self, title: impl AsRef<str>) -> Self {
        self.bold(title).newline().newline()
    }

    /// Returns a reference to the current buffer string without consuming the builder.
    pub fn as_str(&self) -> &str {
        &self.buffer
    }

    /// Consumes the builder and returns the completed formatted message string.
    pub fn build(self) -> String {
        self.buffer
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_formatters() {
        assert_eq!(bold("test"), "*test*");
        assert_eq!(italic("test"), "_test_");
        assert_eq!(strikethrough("test"), "~test~");
        assert_eq!(monospace("test"), "`test`");
        assert_eq!(code_block("code"), "```\ncode\n```");
        assert_eq!(quote("a\nb"), "> a\n> b");
    }

    #[test]
    fn test_lists() {
        assert_eq!(bullet_list(["one", "two"]), "• one\n• two");
        assert_eq!(numbered_list(["one", "two"]), "1. one\n2. two");
    }

    #[test]
    fn test_message_builder() {
        let msg = MessageBuilder::new()
            .header("Title")
            .line("Intro")
            .bold("Bold")
            .text(" ")
            .italic("Italic")
            .newline()
            .bullet("Item 1")
            .numbered(2, "Item 2")
            .quote("Quote")
            .code_block("fn main() {}")
            .build();

        assert!(msg.contains("*Title*\n\nIntro\n*Bold* _Italic_\n• Item 1\n2. Item 2\n> Quote\n```\nfn main() {}\n```\n"));
    }
}
