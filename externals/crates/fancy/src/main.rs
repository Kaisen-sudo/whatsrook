use whatsrook_sdk::{respond, Request};

struct FontStyle {
    key: &'static str,
    name: &'static str,
}

const FONTS: &[FontStyle] = &[
    FontStyle { key: "bold", name: "Bold (Serif)" },
    FontStyle { key: "italic", name: "Italic" },
    FontStyle { key: "bold-italic", name: "Bold Italic" },
    FontStyle { key: "double-struck", name: "Double Struck" },
    FontStyle { key: "script", name: "Script" },
    FontStyle { key: "bold-script", name: "Bold Script" },
    FontStyle { key: "fraktur", name: "Fraktur" },
    FontStyle { key: "bold-fraktur", name: "Bold Fraktur" },
    FontStyle { key: "sans", name: "Sans-Serif" },
    FontStyle { key: "sans-bold", name: "Sans-Serif Bold" },
    FontStyle { key: "sans-italic", name: "Sans-Serif Italic" },
    FontStyle { key: "sans-bold-italic", name: "Sans-Serif Bold Italic" },
    FontStyle { key: "monospace", name: "Monospace / Typewriter" },
    FontStyle { key: "small-caps", name: "Small Caps" },
    FontStyle { key: "inverted", name: "Inverted / Upside Down" },
    FontStyle { key: "circled", name: "Circled Letters" },
    FontStyle { key: "squared", name: "Squared" },
    FontStyle { key: "parenthesized", name: "Parenthesized" },
    FontStyle { key: "fullwidth", name: "Fullwidth / Wide" },
    FontStyle { key: "superscript", name: "Superscript" },
    FontStyle { key: "subscript", name: "Subscript" },
    FontStyle { key: "curved", name: "Curved / Medieval" },
];

fn convert_char(c: char, style: &str) -> String {
    match style {
        "monospace" => {
            if c.is_ascii_lowercase() {
                char::from_u32((c as u32) - ('a' as u32) + 0x1D68A).unwrap_or(c).to_string()
            } else if c.is_ascii_uppercase() {
                char::from_u32((c as u32) - ('A' as u32) + 0x1D670).unwrap_or(c).to_string()
            } else if c.is_ascii_digit() {
                char::from_u32((c as u32) - ('0' as u32) + 0x1D7F6).unwrap_or(c).to_string()
            } else {
                c.to_string()
            }
        }
        "bold" => {
            if c.is_ascii_lowercase() {
                char::from_u32((c as u32) - ('a' as u32) + 0x1D5BA).unwrap_or(c).to_string()
            } else if c.is_ascii_uppercase() {
                char::from_u32((c as u32) - ('A' as u32) + 0x1D5A0).unwrap_or(c).to_string()
            } else if c.is_ascii_digit() {
                char::from_u32((c as u32) - ('0' as u32) + 0x1D7EC).unwrap_or(c).to_string()
            } else {
                c.to_string()
            }
        }
        "italic" => {
            if c == 'h' {
                "ℎ".to_string()
            } else if c.is_ascii_lowercase() {
                char::from_u32((c as u32) - ('a' as u32) + 0x1D434 + 26).unwrap_or(c).to_string()
            } else if c.is_ascii_uppercase() {
                char::from_u32((c as u32) - ('A' as u32) + 0x1D434).unwrap_or(c).to_string()
            } else {
                c.to_string()
            }
        }
        "bold-italic" => {
            if c.is_ascii_lowercase() {
                char::from_u32((c as u32) - ('a' as u32) + 0x1D482).unwrap_or(c).to_string()
            } else if c.is_ascii_uppercase() {
                char::from_u32((c as u32) - ('A' as u32) + 0x1D468).unwrap_or(c).to_string()
            } else {
                c.to_string()
            }
        }
        "double-struck" => {
            if c.is_ascii_lowercase() {
                char::from_u32((c as u32) - ('a' as u32) + 0x1D552).unwrap_or(c).to_string()
            } else if c.is_ascii_uppercase() {
                match c {
                    'C' => "ℂ".to_string(),
                    'H' => "ℍ".to_string(),
                    'N' => "ℕ".to_string(),
                    'P' => "ℙ".to_string(),
                    'Q' => "ℚ".to_string(),
                    'R' => "ℝ".to_string(),
                    'Z' => "ℤ".to_string(),
                    _ => char::from_u32((c as u32) - ('A' as u32) + 0x1D538).unwrap_or(c).to_string(),
                }
            } else if c.is_ascii_digit() {
                char::from_u32((c as u32) - ('0' as u32) + 0x1D7D8).unwrap_or(c).to_string()
            } else {
                c.to_string()
            }
        }
        "script" => {
            if c.is_ascii_lowercase() {
                char::from_u32((c as u32) - ('a' as u32) + 0x1D4EA).unwrap_or(c).to_string()
            } else if c.is_ascii_uppercase() {
                char::from_u32((c as u32) - ('A' as u32) + 0x1D4D0).unwrap_or(c).to_string()
            } else {
                c.to_string()
            }
        }
        "bold-script" => {
            if c.is_ascii_lowercase() {
                char::from_u32((c as u32) - ('a' as u32) + 0x1D4B6).unwrap_or(c).to_string()
            } else if c.is_ascii_uppercase() {
                char::from_u32((c as u32) - ('A' as u32) + 0x1D49C).unwrap_or(c).to_string()
            } else {
                c.to_string()
            }
        }
        "fraktur" => {
            if c.is_ascii_lowercase() {
                char::from_u32((c as u32) - ('a' as u32) + 0x1D520).unwrap_or(c).to_string()
            } else if c.is_ascii_uppercase() {
                match c {
                    'C' => "ℭ".to_string(),
                    'H' => "ℌ".to_string(),
                    'I' => "ℑ".to_string(),
                    'R' => "ℜ".to_string(),
                    'Z' => "ℨ".to_string(),
                    _ => char::from_u32((c as u32) - ('A' as u32) + 0x1D504).unwrap_or(c).to_string(),
                }
            } else {
                c.to_string()
            }
        }
        "bold-fraktur" => {
            if c.is_ascii_lowercase() {
                char::from_u32((c as u32) - ('a' as u32) + 0x1D586).unwrap_or(c).to_string()
            } else if c.is_ascii_uppercase() {
                char::from_u32((c as u32) - ('A' as u32) + 0x1D56C).unwrap_or(c).to_string()
            } else {
                c.to_string()
            }
        }
        "small-caps" => {
            let mapping = [
                ('a', 'ᴀ'), ('b', 'ʙ'), ('c', 'ᴄ'), ('d', 'ᴅ'), ('e', 'ᴇ'),
                ('f', 'ꜰ'), ('g', 'ɢ'), ('h', 'ʜ'), ('i', 'ɪ'), ('j', 'ᴊ'),
                ('k', 'ᴋ'), ('l', 'ʟ'), ('m', 'ᴍ'), ('n', 'ɴ'), ('o', 'ᴏ'),
                ('p', 'ᴘ'), ('q', 'ꞯ'), ('r', 'ʀ'), ('s', 'ꜱ'), ('t', 'ᴛ'),
                ('u', 'ᴜ'), ('v', 'ᴠ'), ('w', 'ᴡ'), ('x', 'x'), ('y', 'ʏ'),
                ('z', 'ᴢ'),
            ];
            let lower = c.to_ascii_lowercase();
            if let Some((_, sc)) = mapping.iter().find(|(orig, _)| *orig == lower) {
                sc.to_string()
            } else {
                c.to_string()
            }
        }
        "circled" => {
            if c.is_ascii_lowercase() {
                char::from_u32((c as u32) - ('a' as u32) + 0x24D0).unwrap_or(c).to_string()
            } else if c.is_ascii_uppercase() {
                char::from_u32((c as u32) - ('A' as u32) + 0x24B6).unwrap_or(c).to_string()
            } else if c.is_ascii_digit() && c != '0' {
                char::from_u32((c as u32) - ('1' as u32) + 0x2460).unwrap_or(c).to_string()
            } else if c == '0' {
                "⓪".to_string()
            } else {
                c.to_string()
            }
        }
        "squared" => {
            if c.is_ascii_uppercase() {
                char::from_u32((c as u32) - ('A' as u32) + 0x1F130).unwrap_or(c).to_string()
            } else if c.is_ascii_lowercase() {
                char::from_u32((c as u32) - ('a' as u32) + 0x1F130).unwrap_or(c).to_string()
            } else {
                c.to_string()
            }
        }
        "fullwidth" => {
            if c.is_ascii_alphanumeric() {
                char::from_u32((c as u32) - ('!' as u32) + 0xFF01).unwrap_or(c).to_string()
            } else {
                c.to_string()
            }
        }
        _ => c.to_string(),
    }
}

fn convert_text(text: &str, style: &str) -> String {
    text.chars().map(|c| convert_char(c, style)).collect()
}

fn main() {
    let req = Request::load();
    let quoted = req.quoted_text().map(|s| s.trim().to_string());
    let query = req.query();

    if query.is_empty() && quoted.is_none() {
        let mut help = format!(
            "*AVAILABLE FONT STYLES (1-{})*\n\n",
            FONTS.len()
        );
        for (i, f) in FONTS.iter().enumerate() {
            let preview = convert_text("WhatsRook Bot", f.key);
            help.push_str(&format!("{}. {} → {}\n", i + 1, f.name, preview));
        }
        help.push_str(&format!(
            "\n*Usage:*\n- {}fancy <number> <text>\n- {}fancy <number> (as reply to a message)\n- Example: `{}fancy 14 Hello World`",
            req.prefix(),
            req.prefix(),
            req.prefix()
        ));
        respond(help);
        return;
    }

    let mut font_idx = 13; // default small-caps (14th, 0-indexed 13)
    let mut text_to_convert = query.clone();

    if !req.args.is_empty() {
        if let Ok(num) = req.args[0].parse::<usize>() {
            if num >= 1 && num <= FONTS.len() {
                font_idx = num - 1;
                let offset = req.args[0].len();
                text_to_convert = req.raw_args[offset..].trim().to_string();
            }
        }
    }

    if text_to_convert.is_empty() {
        if let Some(q_text) = quoted {
            text_to_convert = q_text;
        }
    }

    if text_to_convert.is_empty() {
        respond(format!(
            "Please provide text to convert.\nExample: `{}fancy {} Hello World`",
            req.prefix(),
            font_idx + 1
        ));
        return;
    }

    let style = FONTS[font_idx].key;
    let converted = convert_text(&text_to_convert, style);
    respond(converted);
}
