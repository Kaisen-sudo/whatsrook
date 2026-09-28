use rand::seq::SliceRandom;
use serde::Deserialize;
use whatsrook_sdk::{create_http_client, respond};

#[derive(Deserialize)]
struct DummyQuote {
    #[serde(default)]
    quote: String,
    #[serde(default)]
    author: String,
}

#[derive(Deserialize)]
struct ZenQuote {
    #[serde(default)]
    q: String,
    #[serde(default)]
    a: String,
}

const FALLBACK_QUOTES: &[&str] = &[
    "\"The secret of getting ahead is getting started.\" – Mark Twain",
    "\"It always seems impossible until it's done.\" – Nelson Mandela",
    "\"Do what you can, with what you have, where you are.\" – Theodore Roosevelt",
    "\"In the middle of every difficulty lies opportunity.\" – Albert Einstein",
    "\"Success is not final, failure is not fatal: It is the courage to continue that counts.\" – Winston Churchill",
    "\"Chains of habit are too light to be felt until they are too heavy to be broken.\" – Warren Buffett",
    "\"The only way to do great work is to love what you do.\" – Steve Jobs",
];

fn main() {
    let client = create_http_client(4);

    // Try DummyJSON API
    if let Ok(resp) = client.get("https://dummyjson.com/quotes/random").send() {
        if resp.status().is_success() {
            if let Ok(data) = resp.json::<DummyQuote>() {
                let q = data.quote.trim();
                let a = data.author.trim();
                if !q.is_empty() {
                    if !a.is_empty() {
                        respond(format!("💬 \"{}\" – {}", q, a));
                    } else {
                        respond(format!("💬 \"{}\"", q));
                    }
                    return;
                }
            }
        }
    }

    // Try ZenQuotes API
    if let Ok(resp) = client.get("https://zenquotes.io/api/random").send() {
        if resp.status().is_success() {
            if let Ok(list) = resp.json::<Vec<ZenQuote>>() {
                if let Some(first) = list.first() {
                    let q = first.q.trim();
                    let a = first.a.trim();
                    if !q.is_empty() {
                        if !a.is_empty() {
                            respond(format!("💬 \"{}\" – {}", q, a));
                        } else {
                            respond(format!("💬 \"{}\"", q));
                        }
                        return;
                    }
                }
            }
        }
    }

    // Fallback
    let mut rng = rand::thread_rng();
    let choice = FALLBACK_QUOTES
        .choose(&mut rng)
        .unwrap_or(&FALLBACK_QUOTES[0]);
    respond(format!("💬 {}", choice));
}
