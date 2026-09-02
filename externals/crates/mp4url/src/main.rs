use base64::Engine;
use url::Url;
use whatsrook_sdk::{create_http_client, respond, respond_err, send_video, Request};

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
            "Usage: {}mp4url <direct_video_url>\n\nExample: `{}mp4url https://example.com/video.mp4`",
            req.prefix(),
            req.prefix()
        ));
        return;
    }

    let mut target_url = query.split_whitespace().next().unwrap_or("").to_string();
    target_url = target_url.trim_start_matches('<').trim_end_matches('>').to_string();

    if !target_url.starts_with("http://") && !target_url.starts_with("https://") {
        target_url = format!("https://{}", target_url);
    }

    if let Ok(parsed) = Url::parse(&target_url) {
        if parsed.host_str().map(|h| h.contains('.')).unwrap_or(false) {
            let client = create_http_client(60);
            match client.get(&target_url).send() {
                Ok(resp) => {
                    if !resp.status().is_success() {
                        respond_err(format!("Video server returned status {}", resp.status()));
                    }

                    match resp.bytes() {
                        Ok(bytes) => {
                            if bytes.is_empty() {
                                respond_err("Downloaded empty video payload.");
                            }
                            // Max video size 64 MiB
                            if bytes.len() > 64 * 1024 * 1024 {
                                respond_err(format!(
                                    "Video file is too large ({:.2} MB). Maximum size is 64 MB.",
                                    bytes.len() as f64 / (1024.0 * 1024.0)
                                ));
                            }

                            let b64 = format!(
                                "data:video/mp4;base64,{}",
                                base64::engine::general_purpose::STANDARD.encode(&bytes)
                            );
                            send_video(&b64, Some(&format!("🎬 Video: {}", target_url)));
                        }
                        Err(e) => {
                            respond_err(format!("Failed to download video stream: {}", e));
                        }
                    }
                }
                Err(e) => {
                    respond_err(format!("Network error fetching video: {}", e));
                }
            }
            return;
        }
    }

    respond_err(format!("Invalid video URL `{}`.", target_url));
}
