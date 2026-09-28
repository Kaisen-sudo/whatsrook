use regex::Regex;
use whatsrook_sdk::{create_http_client, respond, respond_err, Request};

struct Article {
    title: String,
    description: String,
    url: String,
}

fn clean_html(text: &str) -> String {
    let re = Regex::new(r"<[^>]*>").unwrap();
    let stripped = re.replace_all(text, "");
    html_escape::decode_html_entities(&stripped)
        .trim()
        .to_string()
}

fn title_case(s: &str) -> String {
    s.split_whitespace()
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn main() {
    let req = Request::load();
    let query = req.query();

    if query.is_empty() {
        let p = req.prefix();
        respond(format!(
            "*AP News Country Headlines*\n\nUsage:\n• {}news <country> (e.g. {}news nigeria, {}news japan, {}news usa, {}news uk)",
            p, p, p, p, p
        ));
        return;
    }

    let country = query.trim().to_lowercase().replace(' ', "-");
    let hub_url = format!("https://apnews.com/hub/{}", country);

    let client = create_http_client(15);
    match client.get(&hub_url).send() {
        Ok(resp) => {
            if resp.status().as_u16() == 404 {
                respond(format!("No news topic hub found for {:?}.", query));
                return;
            }
            if !resp.status().is_success() {
                respond_err(format!("AP News returned status: {}", resp.status()));
            }

            match resp.text() {
                Ok(html) => {
                    let promo_regex = Regex::new(
                        r#"(?s)<div class="PagePromo"[^>]*>(.*?)</div>\s*</div>\s*</div>"#,
                    )
                    .unwrap();
                    let link_regex =
                        Regex::new(r#"href="(https?://apnews\.com/article/[^"]+|/article/[^"]+)""#)
                            .unwrap();
                    let title_regex = Regex::new(r#"(?s)<h3 class="PagePromo-title"[^>]*>.*?<span class="PagePromoContentIcons-text">(.*?)</span>"#).unwrap();
                    let alt_title_regex =
                        Regex::new(r#"(?s)<h3 class="PagePromo-title"[^>]*>.*?<a[^>]*>(.*?)</a>"#)
                            .unwrap();
                    let desc_regex = Regex::new(r#"(?s)<div class="PagePromo-description"[^>]*>.*?<span class="PagePromoContentIcons-text">(.*?)</span>"#).unwrap();

                    let mut articles = Vec::new();
                    let mut seen = std::collections::HashSet::new();

                    for cap in promo_regex.captures_iter(&html) {
                        let block = &cap[1];
                        let mut url = String::new();
                        if let Some(link_cap) = link_regex.captures(block) {
                            url = link_cap[1].to_string();
                            if url.starts_with('/') {
                                url = format!("https://apnews.com{}", url);
                            }
                        }
                        if url.is_empty() || seen.contains(&url) {
                            continue;
                        }

                        let mut title = String::new();
                        if let Some(t_cap) = title_regex.captures(block) {
                            title = clean_html(&t_cap[1]);
                        } else if let Some(t_cap) = alt_title_regex.captures(block) {
                            title = clean_html(&t_cap[1]);
                        }

                        if title.is_empty() {
                            continue;
                        }

                        let mut desc = String::new();
                        if let Some(d_cap) = desc_regex.captures(block) {
                            desc = clean_html(&d_cap[1]);
                        }

                        seen.insert(url.clone());
                        articles.push(Article {
                            title,
                            description: desc,
                            url,
                        });

                        if articles.len() >= 5 {
                            break;
                        }
                    }

                    if articles.is_empty() {
                        respond(format!("No recent news articles found for {:?}.", query));
                        return;
                    }

                    let display_country = title_case(&country.replace('-', " "));
                    let mut out = format!("*AP News - {}*\n\n", display_country);
                    for (i, art) in articles.iter().enumerate() {
                        out.push_str(&format!("{}. *{}*\n", i + 1, art.title));
                        if !art.description.is_empty() {
                            out.push_str(&format!("   {}\n", art.description));
                        }
                        out.push_str(&format!("   {}\n\n", art.url));
                    }

                    respond(out.trim());
                }
                Err(err) => {
                    respond_err(format!("Failed to read news response: {}", err));
                }
            }
        }
        Err(err) => {
            respond_err(format!("Network error fetching news: {}", err));
        }
    }
}
