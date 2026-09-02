use base64::Engine;
use whatsrook_sdk::{create_http_client, respond, respond_err, send_image, Request};

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
            "Usage: {}qr <text or url> (or reply to a message)",
            req.prefix()
        ));
        return;
    }

    let encoded_query: String =
        url::form_urlencoded::byte_serialize(query.as_bytes()).collect();

    // Fetch high resolution QR Code image from primary QR service
    let qr_url = format!(
        "https://api.qrserver.com/v1/create-qr-code/?size=600x600&margin=15&data={}",
        encoded_query
    );

    let client = create_http_client(15);
    match client.get(&qr_url).send() {
        Ok(resp) => {
            if !resp.status().is_success() {
                respond_err(format!(
                    "QR Code service returned status: {}",
                    resp.status()
                ));
            }

            match resp.bytes() {
                Ok(bytes) => {
                    let b64 = format!(
                        "data:image/png;base64,{}",
                        base64::engine::general_purpose::STANDARD.encode(&bytes)
                    );
                    send_image(&b64, Some("QR Code Generated"));
                }
                Err(err) => {
                    respond_err(format!("Error reading QR Code image: {}", err));
                }
            }
        }
        Err(err) => {
            // Fallback to quickchart QR API
            let fallback_url = format!(
                "https://quickchart.io/qr?size=600&margin=2&text={}",
                encoded_query
            );
            if let Ok(resp) = client.get(&fallback_url).send() {
                if resp.status().is_success() {
                    if let Ok(bytes) = resp.bytes() {
                        let b64 = format!(
                            "data:image/png;base64,{}",
                            base64::engine::general_purpose::STANDARD.encode(&bytes)
                        );
                        send_image(&b64, Some("QR Code Generated"));
                        return;
                    }
                }
            }
            respond_err(format!("Network error generating QR code: {}", err));
        }
    }
}
