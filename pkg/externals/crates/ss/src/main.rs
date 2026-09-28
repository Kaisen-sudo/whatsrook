use base64::Engine;
use serde::Deserialize;
use url::Url;
use whatsrook_sdk::{create_http_client, respond, respond_err, send_image, Request};

#[derive(Deserialize)]
struct MicroLinkResponse {
    #[serde(default)]
    data: Option<MicroLinkData>,
}

#[derive(Deserialize)]
struct MicroLinkData {
    #[serde(default)]
    screenshot: Option<MicroLinkScreenshot>,
}

#[derive(Deserialize)]
struct MicroLinkScreenshot {
    #[serde(default)]
    url: String,
}

fn main() {
    let req = Request::load();
    let mut query = req.query();

    if query.is_empty() {
        if let Some(q_text) = req.quoted_text() {
            query = q_text.trim().to_string();
        }
    }

    if query.is_empty() {
        respond(format!(
            "Usage: {}ss <URL>\n\nExamples:\n- {}ss https://google.com\n- Reply to a message containing a URL with {}ss",
            req.prefix(),
            req.prefix(),
            req.prefix()
        ));
        return;
    }

    // Extract first URL-like token from query
    let mut target_url = query.split_whitespace().next().unwrap_or("").to_string();
    target_url = target_url
        .trim_start_matches('<')
        .trim_end_matches('>')
        .to_string();

    if !target_url.starts_with("http://") && !target_url.starts_with("https://") {
        target_url = format!("https://{}", target_url);
    }

    if let Ok(parsed) = Url::parse(&target_url) {
        if parsed.host_str().map(|h| h.contains('.')).unwrap_or(false) {
            let client = create_http_client(20);

            // 1. Try thum.io first
            let thum_url = format!(
                "https://image.thum.io/get/width/1280/crop/800/noanimate/{}",
                target_url
            );
            if let Ok(resp) = client.get(&thum_url).send() {
                if resp.status().is_success() {
                    if let Ok(bytes) = resp.bytes() {
                        if bytes.len() > 5000 {
                            let b64 = format!(
                                "data:image/jpeg;base64,{}",
                                base64::engine::general_purpose::STANDARD.encode(&bytes)
                            );
                            send_image(&b64, Some(&format!("Screenshot: {}", target_url)));
                            return;
                        }
                    }
                }
            }

            // 2. Try microlink.io screenshot API
            let microlink_url = format!(
                "https://api.microlink.io/?url={}&screenshot=true",
                url::form_urlencoded::byte_serialize(target_url.as_bytes()).collect::<String>()
            );
            if let Ok(resp) = client.get(&microlink_url).send() {
                if resp.status().is_success() {
                    if let Ok(json) = resp.json::<MicroLinkResponse>() {
                        if let Some(data) = json.data {
                            if let Some(screenshot) = data.screenshot {
                                if !screenshot.url.is_empty() {
                                    if let Ok(img_resp) = client.get(&screenshot.url).send() {
                                        if let Ok(img_bytes) = img_resp.bytes() {
                                            if img_bytes.len() > 5000 {
                                                let b64 = format!(
                                                    "data:image/jpeg;base64,{}",
                                                    base64::engine::general_purpose::STANDARD
                                                        .encode(&img_bytes)
                                                );
                                                send_image(
                                                    &b64,
                                                    Some(&format!("Screenshot: {}", target_url)),
                                                );
                                                return;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            respond_err(format!(
                "Failed to capture screenshot for `{}`. Please check the URL and try again.",
                target_url
            ));
        }
    }

    respond_err(format!(
        "Invalid URL `{}`. Please specify a valid web address (e.g. `https://github.com`).",
        target_url
    ));
}
