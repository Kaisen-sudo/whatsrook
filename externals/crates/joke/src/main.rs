use rand::seq::SliceRandom;
use serde::Deserialize;
use whatsrook_sdk::{create_http_client, respond};

#[derive(Deserialize)]
struct DadJoke {
    #[serde(default)]
    joke: String,
}

#[derive(Deserialize)]
struct OfficialJoke {
    #[serde(default)]
    setup: String,
    #[serde(default)]
    punchline: String,
}

const FALLBACK_JOKES: &[&str] = &[
    "Why don't scientists trust atoms? Because they make up everything!",
    "Why did the scarecrow win an award? Because he was outstanding in his field!",
    "What do you call fake spaghetti? An impasta!",
    "Why do programmers prefer dark mode? Because light attracts bugs!",
    "How do you organize a space party? You planet!",
    "What do you call a pig that knows karate? A pork chop!",
    "Why don't eggs tell jokes? They'd crack each other up!",
];

fn main() {
    let client = create_http_client(4);

    // Try ICanHazDadJoke API
    if let Ok(resp) = client
        .get("https://icanhazdadjoke.com/")
        .header("Accept", "application/json")
        .send()
    {
        if resp.status().is_success() {
            if let Ok(data) = resp.json::<DadJoke>() {
                let trimmed = data.joke.trim();
                if !trimmed.is_empty() {
                    respond(format!("😂 {}", trimmed));
                    return;
                }
            }
        }
    }

    // Try Official Joke API
    if let Ok(resp) = client
        .get("https://official-joke-api.appspot.com/random_joke")
        .send()
    {
        if resp.status().is_success() {
            if let Ok(data) = resp.json::<OfficialJoke>() {
                let setup = data.setup.trim();
                let punchline = data.punchline.trim();
                if !setup.is_empty() && !punchline.is_empty() {
                    respond(format!("😂 {}\n\n{}", setup, punchline));
                    return;
                }
            }
        }
    }

    // Fallback
    let mut rng = rand::thread_rng();
    let choice = FALLBACK_JOKES
        .choose(&mut rng)
        .unwrap_or(&FALLBACK_JOKES[0]);
    respond(format!("😂 {}", choice));
}
