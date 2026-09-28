use serde::Deserialize;
use std::thread;
use std::time::{Duration, Instant};
use whatsrook_sdk::{
    create_http_client, respond_err, send_done, send_edit_live, send_reply_live, HttpClient,
};

const TICK: Duration = Duration::from_millis(1500);
const MAX_DURATION: Duration = Duration::from_secs(5 * 60);

#[derive(Deserialize, Default)]
struct WatcherGuruResponse {
    #[serde(default)]
    bitcoin_price: BitcoinPriceInfo,
    #[serde(default)]
    current: BlockInfo,
    #[serde(default)]
    target: TargetBlockInfo,
}

#[derive(Deserialize, Default)]
struct BitcoinPriceInfo {
    #[serde(default)]
    price_usd: f64,
    #[serde(default)]
    price_change_24h: f64,
}

#[derive(Deserialize, Default)]
struct BlockInfo {
    #[serde(default)]
    block_number: i64,
}

#[derive(Deserialize, Default)]
struct TargetBlockInfo {
    #[serde(default)]
    block_number: i64,
}

#[derive(Deserialize)]
struct BinancePriceResponse {
    #[serde(default)]
    price: String,
}

fn format_commas(n: i64) -> String {
    let s = n.abs().to_string();
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len();
    let mut result = String::new();
    for (i, &c) in chars.iter().enumerate() {
        if i > 0 && (len - i).is_multiple_of(3) {
            result.push(',');
        }
        result.push(c);
    }
    if n < 0 {
        format!("-{}", result)
    } else {
        result
    }
}

fn format_usd(price: f64) -> String {
    let int_part = price as i64;
    let dec = ((price - int_part as f64) * 100.0).round() as i64;
    let dec = if dec >= 100 { 99 } else { dec.max(0) };
    format!("${}.{:02}", format_commas(int_part), dec)
}

fn format_change(change: f64) -> String {
    if change > 0.0 {
        format!("📈 +{:.2}%", change)
    } else if change < 0.0 {
        format!("📉 {:.2}%", change)
    } else {
        "➡️ 0.00%".to_string()
    }
}

fn build_message(data: &WatcherGuruResponse, prefix: &str, status: &str) -> String {
    let price_str = format_usd(data.bitcoin_price.price_usd);
    let change_str = if data.bitcoin_price.price_change_24h != 0.0 {
        format!(" ({})", format_change(data.bitcoin_price.price_change_24h))
    } else {
        String::new()
    };

    let mut out = format!("₿ *Bitcoin (BTC)*\n\n*Price:* {}{}", price_str, change_str);

    if data.current.block_number > 0 {
        out.push_str(&format!(
            "\n*Current Block:* {}",
            format_commas(data.current.block_number)
        ));
    }

    if data.target.block_number > 0 {
        out.push_str(&format!(
            "\n*Halving Block:* {}",
            format_commas(data.target.block_number)
        ));

        if data.current.block_number > 0 {
            let remaining = data.target.block_number - data.current.block_number;
            if remaining > 0 {
                let minutes = remaining * 10;
                let days = minutes / (60 * 24);
                let hours = (minutes % (60 * 24)) / 60;
                out.push_str(&format!(
                    "\n*Blocks Remaining:* {}",
                    format_commas(remaining)
                ));
                if days > 0 {
                    out.push_str(&format!(
                        "\n*Est. Time:* ~{} days {} hrs",
                        format_commas(days),
                        hours
                    ));
                } else if hours > 0 {
                    let mins = minutes % 60;
                    out.push_str(&format!("\n*Est. Time:* ~{} hrs {} min", hours, mins));
                } else {
                    out.push_str(&format!("\n*Est. Time:* ~{} min", minutes % 60));
                }
            } else {
                out.push_str("\n*Halving:* ✅ Completed");
            }
        }
    }

    out.push_str(&format!("\n\n_{}_", status));
    out.push_str(&format!("\n_{}markets for more_", prefix));
    out
}

fn fetch_data(client: &HttpClient) -> Option<WatcherGuruResponse> {
    if let Ok(resp) = client
        .get("https://api.watcher.guru/bitcoinhalving/predictions")
        .send()
    {
        if resp.status().is_success() {
            if let Ok(data) = resp.json::<WatcherGuruResponse>() {
                if data.bitcoin_price.price_usd > 0.0 {
                    return Some(data);
                }
            }
        }
    }
    None
}

fn fetch_fallback_price(client: &HttpClient) -> Option<f64> {
    if let Ok(resp) = client
        .get("https://api.binance.com/api/v3/ticker/price?symbol=BTCUSDT")
        .send()
    {
        if resp.status().is_success() {
            if let Ok(data) = resp.json::<BinancePriceResponse>() {
                return data.price.parse::<f64>().ok();
            }
        }
    }
    None
}

fn main() {
    let req = whatsrook_sdk::Request::load_streaming();
    let prefix = req.prefix().to_string();
    let client = create_http_client(10);

    // Fetch initial data
    let initial_data = match fetch_data(&client) {
        Some(d) => d,
        None => {
            // Try Binance fallback — can't do live edits without halving data,
            // but still give useful price info
            match fetch_fallback_price(&client) {
                Some(price) => {
                    let text = format!(
                        "₿ *Bitcoin (BTC)*\n\n*Price:* {}\n\n_Powered by Binance · {}markets for more_",
                        format_usd(price),
                        prefix
                    );
                    // Plain respond — no live session
                    println!("{}", text.trim());
                    return;
                }
                None => respond_err("Failed to fetch Bitcoin market data."),
            }
        }
    };

    let status = format!("Use {}btc stop to end live tracking", prefix);
    let initial_text = build_message(&initial_data, &prefix, &status);

    // Send initial message and get msg_id for live edits
    let msg_id = match send_reply_live(&initial_text) {
        Some(id) => id,
        None => {
            // Fallback: just send a plain reply (e.g. in test mode)
            println!("{}", initial_text.trim());
            return;
        }
    };

    let start = Instant::now();

    loop {
        thread::sleep(TICK);

        if start.elapsed() >= MAX_DURATION {
            // Session timed out
            if let Some(data) = fetch_data(&client) {
                let final_text =
                    build_message(&data, &prefix, "⏱️ Live tracking ended (5m timeout).");
                send_edit_live(&msg_id, &final_text);
            }
            break;
        }

        if let Some(data) = fetch_data(&client) {
            let updated = build_message(&data, &prefix, &status);
            send_edit_live(&msg_id, &updated);
        }
    }

    send_done();
}
