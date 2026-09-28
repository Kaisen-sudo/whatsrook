use serde::{Deserialize, Serialize};
use whatsrook_sdk::{create_http_client, respond, respond_err, Request};

#[derive(Serialize)]
struct WhyRequest {
    action: String,
    query: String,
}

#[derive(Deserialize)]
struct WhyResponse {
    #[serde(default)]
    answer: String,
    #[serde(default)]
    pulls: Vec<WhyPull>,
    #[serde(default)]
    error: String,
}

#[derive(Deserialize)]
struct WhyPull {
    #[serde(default)]
    label: String,
    #[serde(default)]
    query: String,
}

fn main() {
    let req = Request::load();
    let query = req.query();

    if query.is_empty() {
        respond("*Why.com AI Search*\n\nUsage:\n• why <question or topic>");
        return;
    }

    let payload = WhyRequest {
        action: "answer".to_string(),
        query: query.clone(),
    };

    let client = create_http_client(30);
    match client
        .post("https://why.com/api/ultimate-search")
        .header("Content-Type", "application/json")
        .header("Origin", "https://why.com")
        .header("Referer", "https://why.com/")
        .json(&payload)
        .send()
    {
        Ok(resp) => {
            if !resp.status().is_success() {
                respond_err(format!("Why.com API returned status: {}", resp.status()));
            }

            match resp.json::<WhyResponse>() {
                Ok(data) => {
                    if !data.error.is_empty() {
                        respond_err(format!("Why.com error: {}", data.error));
                    }

                    let answer = data.answer.trim();
                    if answer.is_empty() {
                        respond("No answer found.");
                        return;
                    }

                    let mut out = format!("💡 *Why.com Analysis*\n\n{}", answer);

                    let relevant_pulls: Vec<&WhyPull> = data
                        .pulls
                        .iter()
                        .filter(|p| !p.label.is_empty() || !p.query.is_empty())
                        .take(3)
                        .collect();

                    if !relevant_pulls.is_empty() {
                        out.push_str("\n\n*Related Explorations:*");
                        for pull in relevant_pulls {
                            let text = if !pull.label.is_empty() {
                                &pull.label
                            } else {
                                &pull.query
                            };
                            out.push_str(&format!("\n• {}", text));
                        }
                    }

                    respond(out);
                }
                Err(err) => {
                    respond_err(format!("Failed to parse Why.com response: {}", err));
                }
            }
        }
        Err(err) => {
            respond_err(format!("Network error contacting why.com: {}", err));
        }
    }
}
