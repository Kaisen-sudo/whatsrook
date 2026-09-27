use rand::seq::SliceRandom;
use serde::Deserialize;
use whatsrook_sdk::{create_http_client, respond};

#[derive(Deserialize)]
struct UselessFactResponse {
    #[serde(default)]
    text: String,
}

#[derive(Deserialize)]
struct CatFactResponse {
    #[serde(default)]
    fact: String,
}

const FALLBACK_FACTS: &[&str] = &[
    "Honey never spoils; archaeologists have found 3,000-year-old edible honey in Egyptian tombs.",
    "Octopuses have three hearts and blue blood.",
    "Bananas are naturally slightly radioactive because they are rich in potassium.",
    "Venus is the only planet in our solar system that rotates clockwise.",
    "A day on Venus is longer than a year on Venus.",
    "Wombat poop is cube-shaped to keep it from rolling away.",
    "Sharks existed before trees.",
    "The U.S. bought Alaska for 2 cents an acre from Russia.",
    "A flock of crows is known as a murder.",
    "There are more trees on Earth than stars in the Milky Way galaxy.",
];

fn main() {
    let client = create_http_client(4);

    // Try UselessFacts API
    if let Ok(resp) = client
        .get("https://uselessfacts.jsph.pl/api/v2/facts/random")
        .send()
    {
        if resp.status().is_success() {
            if let Ok(data) = resp.json::<UselessFactResponse>() {
                let trimmed = data.text.trim();
                if !trimmed.is_empty() {
                    respond(format!("💡 *Fact:* {}", trimmed));
                    return;
                }
            }
        }
    }

    // Try CatFact API
    if let Ok(resp) = client.get("https://catfact.ninja/fact").send() {
        if resp.status().is_success() {
            if let Ok(data) = resp.json::<CatFactResponse>() {
                let trimmed = data.fact.trim();
                if !trimmed.is_empty() {
                    respond(format!("💡 *Fact:* {}", trimmed));
                    return;
                }
            }
        }
    }

    // Fallback
    let mut rng = rand::thread_rng();
    let choice = FALLBACK_FACTS
        .choose(&mut rng)
        .unwrap_or(&FALLBACK_FACTS[0]);
    respond(format!("💡 *Fact:* {}", choice));
}
