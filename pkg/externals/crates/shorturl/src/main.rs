use url::form_urlencoded;
use whatsrook_sdk::{create_http_client, respond, respond_err, Request};

fn main() {
    let req = Request::load();
    let raw = req.query();

    if raw.is_empty() {
        respond("Usage: shorturl [url]");
        return;
    }

    let target_url = if !raw.starts_with("http://") && !raw.starts_with("https://") {
        format!("https://{}", raw)
    } else {
        raw.clone()
    };

    let encoded: String = form_urlencoded::byte_serialize(target_url.as_bytes()).collect();
    let client = create_http_client(10);

    // Try TinyURL first
    let tiny_url = format!("https://tinyurl.com/api-create.php?url={}", encoded);
    if let Ok(resp) = client.get(&tiny_url).send() {
        if resp.status().is_success() {
            if let Ok(short) = resp.text() {
                let trimmed = short.trim();
                if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
                    respond(format!("Shortened URL: {}", trimmed));
                    return;
                }
            }
        }
    }

    // Fallback: is.gd
    let isgd_url = format!("https://is.gd/create.php?format=simple&url={}", encoded);
    if let Ok(resp) = client.get(&isgd_url).send() {
        if resp.status().is_success() {
            if let Ok(short) = resp.text() {
                let trimmed = short.trim();
                if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
                    respond(format!("Shortened URL: {}", trimmed));
                    return;
                }
            }
        }
    }

    respond_err("Failed to shorten URL. Please check if the URL is valid.");
}
