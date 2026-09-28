use base64::Engine;
use whatsrook_sdk::{create_http_client, respond, respond_err, send_audio, Request};

fn is_known_language_code(code: &str) -> bool {
    let supported = [
        "af", "ar", "bg", "bn", "bs", "ca", "cs", "da", "de", "el", "en", "es", "et", "fi", "fr",
        "gu", "hi", "hr", "hu", "id", "is", "it", "ja", "jv", "kn", "ko", "la", "lv", "ml", "mr",
        "ms", "my", "ne", "nl", "no", "pl", "pt", "ro", "ru", "si", "sk", "sq", "sr", "su", "sv",
        "sw", "ta", "te", "th", "tl", "tr", "uk", "ur", "vi", "zh", "zh-cn", "zh-tw",
    ];
    supported.contains(&code)
}

fn main() {
    let req = Request::load();
    let mut text_to_speak = req.query();

    if text_to_speak.is_empty() {
        if let Some(q_text) = req.quoted_text() {
            text_to_speak = q_text.trim().to_string();
        }
    }

    if text_to_speak.is_empty() {
        respond(format!(
            "Usage: {}tts <text> or {}tts <lang_code> <text>\n\nExamples:\n• {}tts Hello world!\n• {}tts es Hola, ¿cómo estás?\n• {}tts fr Bonjour tout le monde",
            req.prefix(),
            req.prefix(),
            req.prefix(),
            req.prefix(),
            req.prefix()
        ));
        return;
    }

    let mut lang = "en";
    if !req.args.is_empty() {
        let first_word = req.args[0].to_lowercase();
        if first_word.len() >= 2
            && first_word.len() <= 5
            && is_known_language_code(&first_word)
            && req.args.len() > 1
        {
            lang = Box::leak(first_word.into_boxed_str());
            let offset = req.args[0].len();
            text_to_speak = req.raw_args[offset..].trim().to_string();
        }
    }

    if text_to_speak.is_empty() {
        respond("Please provide text to convert to speech.");
        return;
    }

    // Limit text length to prevent abuse
    if text_to_speak.chars().count() > 500 {
        text_to_speak = text_to_speak.chars().take(500).collect();
    }

    let encoded_text: String =
        url::form_urlencoded::byte_serialize(text_to_speak.as_bytes()).collect();
    let tts_url = format!(
        "https://translate.google.com/translate_tts?ie=UTF-8&q={}&tl={}&client=tw-ob",
        encoded_text, lang
    );

    let client = create_http_client(15);
    match client
        .get(&tts_url)
        .header("Referer", "https://translate.google.com/")
        .send()
    {
        Ok(resp) => {
            if !resp.status().is_success() {
                respond_err(format!(
                    "Google TTS service returned status: {}",
                    resp.status()
                ));
            }

            match resp.bytes() {
                Ok(bytes) => {
                    if bytes.is_empty() {
                        respond_err("Received empty audio stream from Google TTS.");
                    }
                    let b64 = format!(
                        "data:audio/mp3;base64,{}",
                        base64::engine::general_purpose::STANDARD.encode(&bytes)
                    );
                    send_audio(&b64, true);
                }
                Err(err) => {
                    respond_err(format!("Failed to read audio bytes: {}", err));
                }
            }
        }
        Err(err) => {
            respond_err(format!("Network error contacting Google TTS: {}", err));
        }
    }
}
