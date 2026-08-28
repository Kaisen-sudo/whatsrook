use serde::Deserialize;
use whatsrook_sdk::{create_http_client, respond, respond_err};

#[derive(Deserialize)]
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

fn format_number_with_commas(n: i64) -> String {
    let s = n.abs().to_string();
    let mut result = String::new();
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len();

    for (i, &c) in chars.iter().enumerate() {
        if i > 0 && (len - i) % 3 == 0 {
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

fn format_usd_price(price: f64) -> String {
    let int_part = price as i64;
    let dec_part = ((price - int_part as f64) * 100.0).round() as i64;
    let dec_clamped = if dec_part >= 100 { 99 } else { dec_part.max(0) };
    format!(
        "${}.{:02}",
        format_number_with_commas(int_part),
        dec_clamped
    )
}

fn main() {
    let client = create_http_client(8);

    // Try Watcher Guru API
    if let Ok(resp) = client
        .get("https://api.watcher.guru/bitcoinhalving/predictions")
        .send()
    {
        if resp.status().is_success() {
            if let Ok(data) = resp.json::<WatcherGuruResponse>() {
                if data.bitcoin_price.price_usd > 0.0 {
                    let mut out = format!(
                        "*Bitcoin Price:* {}",
                        format_usd_price(data.bitcoin_price.price_usd)
                    );
                    if data.current.block_number > 0 {
                        out.push_str(&format!(
                            "\n*Current Block:* {}",
                            format_number_with_commas(data.current.block_number)
                        ));
                    }
                    if data.target.block_number > 0 {
                        out.push_str(&format!(
                            "\n*Target Block:* {}",
                            format_number_with_commas(data.target.block_number)
                        ));
                    }
                    respond(out);
                    return;
                }
            }
        }
    }

    // Fallback: Binance Public API
    if let Ok(resp) = client
        .get("https://api.binance.com/api/v3/ticker/price?symbol=BTCUSDT")
        .send()
    {
        if resp.status().is_success() {
            if let Ok(data) = resp.json::<BinancePriceResponse>() {
                if let Ok(price) = data.price.parse::<f64>() {
                    respond(format!("*Bitcoin Price:* {}", format_usd_price(price)));
                    return;
                }
            }
        }
    }

    respond_err("Failed to fetch Bitcoin market data.");
}
