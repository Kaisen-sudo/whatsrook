use serde::Deserialize;
use url::form_urlencoded;
use whatsrook_sdk::{create_http_client, respond, respond_err, Request};

#[derive(Deserialize)]
struct UrbanResponse {
    #[serde(default)]
    list: Vec<UrbanEntry>,
}

#[derive(Deserialize)]
struct UrbanEntry {
    #[serde(default)]
    word: String,
    #[serde(default)]
    definition: String,
    #[serde(default)]
    example: String,
    #[serde(default)]
    author: String,
}

fn main() {
    let req = Request::load();
    let query = req.query();

    if query.is_empty() {
        respond("Usage: urban [term]");
        return;
    }

    let encoded_query: String = form_urlencoded::byte_serialize(query.as_bytes()).collect();
    let url = format!(
        "https://api.urbandictionary.com/v0/define?term={}",
        encoded_query
    );

    let client = create_http_client(10);
    match client.get(&url).send() {
        Ok(resp) => {
            if !resp.status().is_success() {
                respond_err(format!(
                    "Urban Dictionary returned status: {}",
                    resp.status()
                ));
            }

            match resp.json::<UrbanResponse>() {
                Ok(data) => {
                    if data.list.is_empty() {
                        respond(format!(
                            "Could not find Urban Dictionary definition for {:?}.",
                            query
                        ));
                        return;
                    }

                    let first = &data.list[0];
                    let clean_def = first.definition.replace('[', "").replace(']', "");
                    let clean_example = first.example.replace('[', "").replace(']', "");

                    let mut out = format!(
                        "*Urban Dictionary: {}*\n\n*Definition:*\n{}",
                        first.word,
                        clean_def.trim()
                    );

                    if !clean_example.trim().is_empty() {
                        out.push_str("\n\n*Example:*\n");
                        out.push_str(clean_example.trim());
                    }

                    if !first.author.trim().is_empty() {
                        out.push_str("\n\nAuthor: ");
                        out.push_str(first.author.trim());
                    }

                    respond(out);
                }
                Err(err) => {
                    respond_err(format!(
                        "Failed to parse Urban Dictionary response: {}",
                        err
                    ));
                }
            }
        }
        Err(err) => {
            respond_err(format!("Network error: {}", err));
        }
    }
}
