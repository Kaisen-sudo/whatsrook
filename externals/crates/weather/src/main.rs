use url::form_urlencoded;
use whatsrook_sdk::{create_http_client, respond, respond_err, Request};

fn main() {
    let req = Request::load();
    let query = req.query();

    if query.is_empty() {
        respond("Usage: weather [city/town]");
        return;
    }

    let encoded_query: String = form_urlencoded::byte_serialize(query.as_bytes()).collect();
    let url = format!("https://wttr.in/{}?format=4", encoded_query);

    let client = create_http_client(10);
    // wttr.in returns plain text when User-Agent is curl
    match client.get(&url).header("User-Agent", "curl/8.0.0").send() {
        Ok(resp) => {
            if !resp.status().is_success() {
                respond_err(format!(
                    "Weather service returned status: {}",
                    resp.status()
                ));
            }

            match resp.text() {
                Ok(text) => {
                    let trimmed = text.trim();
                    if trimmed.is_empty() || trimmed.contains("Unknown location") {
                        respond(format!("Could not find weather info for {:?}.", query));
                    } else {
                        respond(trimmed);
                    }
                }
                Err(err) => {
                    respond_err(format!("Error reading weather response: {}", err));
                }
            }
        }
        Err(err) => {
            respond_err(format!("Network error fetching weather: {}", err));
        }
    }
}
