use regex::Regex;
use whatsrook_sdk::{create_http_client, respond, respond_err};

fn clean_html(text: &str) -> String {
    let re = Regex::new(r"<[^>]*>").unwrap();
    let stripped = re.replace_all(text, "");
    html_escape::decode_html_entities(&stripped)
        .trim()
        .to_string()
}

fn main() {
    let client = create_http_client(15);

    // 1. Fetch homepage to get latest post link
    let home_html = match client.get("https://wabetainfo.com/").send() {
        Ok(resp) if resp.status().is_success() => resp.text().unwrap_or_default(),
        Ok(resp) => respond_err(format!("WABetaInfo returned status: {}", resp.status())),
        Err(err) => respond_err(format!("Network error reaching WABetaInfo: {}", err)),
    };

    let post_link_regex =
        Regex::new(r#"<a[^>]*href=["'](https?://wabetainfo\.com/[^"'/]+/)["']"#).unwrap();
    let mut article_url = String::new();

    for cap in post_link_regex.captures_iter(&home_html) {
        let url = &cap[1];
        if !url.contains("/android/")
            && !url.contains("/ios/")
            && !url.contains("/news/")
            && !url.contains("/about/")
            && !url.contains("/disclaimer/")
            && !url.contains("/privacy-policy/")
            && !url.contains("/contact-us/")
            && !url.contains("/page/")
        {
            article_url = url.to_string();
            break;
        }
    }

    if article_url.is_empty() {
        respond_err("Failed to extract latest article link from WABetaInfo.");
    }

    // 2. Fetch article page
    let article_html = match client.get(&article_url).send() {
        Ok(resp) if resp.status().is_success() => resp.text().unwrap_or_default(),
        Ok(resp) => respond_err(format!("Article page returned status: {}", resp.status())),
        Err(err) => respond_err(format!("Failed to fetch article: {}", err)),
    };

    // Extract title
    let title_regex = Regex::new(
        r#"(?s)<div[^>]*class=["'][^"']*entry-title[^"']*["'][^>]*>.*?<h1[^>]*>(.*?)</h1>"#,
    )
    .unwrap();
    let h1_fallback = Regex::new(r#"(?s)<h1[^>]*>(.*?)</h1>"#).unwrap();

    let title = if let Some(cap) = title_regex.captures(&article_html) {
        clean_html(&cap[1])
    } else if let Some(cap) = h1_fallback.captures(&article_html) {
        clean_html(&cap[1])
    } else {
        "WABetaInfo Update".to_string()
    };

    // Extract article content paragraphs
    let p_regex = Regex::new(r#"(?s)<p[^>]*>(.*?)</p>"#).unwrap();
    let mut paragraphs = Vec::new();

    // Look within entry-content / kenta-article-content
    let content_block = if let Some(idx) = article_html.find("entry-content") {
        &article_html[idx..]
    } else {
        &article_html
    };

    for cap in p_regex.captures_iter(content_block) {
        let txt = clean_html(&cap[1]);
        if txt.len() > 30
            && !txt.eq_ignore_ascii_case("ADVERTISEMENT")
            && !txt.starts_with("Connect with WABetaInfo")
            && !txt.starts_with("Follow us on")
        {
            paragraphs.push(txt);
            if paragraphs.len() >= 6 {
                break;
            }
        }
    }

    let mut out = format!("*{}*\n\n", title);
    if !paragraphs.is_empty() {
        out.push_str(&paragraphs.join("\n\n"));
    } else {
        out.push_str("Read more at: ");
        out.push_str(&article_url);
    }

    respond(out.trim());
}
