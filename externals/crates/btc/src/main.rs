use serde::Deserialize;
use whatsrook_sdk::{create_http_client, respond, respond_err};

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

fn format_number_with_commas(n: i64) -> String {
    let s = n.abs().to_string();
    let mut result = String::new();
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len();

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

fn format_change(change: f64) -> String {
    if change > 0.0 {
        format!("📈 +{:.2}%", change)
    } else if change < 0.0 {
        format!("📉 {:.2}%", change)
    } else {
        "➡️ 0.00%".to_string()
    }
}

fn main() {
    let req = whatsrook_sdk::Request::load();
    let client = create_http_client(10);

    // Try Watcher Guru API (price + halving)
    if let Ok(resp) = client
        .get("https://api.watcher.guru/bitcoinhalving/predictions")
        .send()
    {
        if resp.status().is_success() {
            if let Ok(data) = resp.json::<WatcherGuruResponse>() {
                if data.bitcoin_price.price_usd > 0.0 {
                    let price_str = format_usd_price(data.bitcoin_price.price_usd);
                    let change_str = if data.bitcoin_price.price_change_24h != 0.0 {
                        format!(" ({})", format_change(data.bitcoin_price.price_change_24h))
                    } else {
                        String::new()
                    };

                    let mut out =
                        format!("₿ *Bitcoin (BTC)*\n\n*Price:* {}{}", price_str, change_str);

                    if data.current.block_number > 0 {
                        out.push_str(&format!(
                            "\n*Current Block:* {}",
                            format_number_with_commas(data.current.block_number)
                        ));
                    }

                    if data.target.block_number > 0 {
                        out.push_str(&format!(
                            "\n*Halving Block:* {}",
                            format_number_with_commas(data.target.block_number)
                        ));

                        if data.current.block_number > 0 {
                            let remaining = data.target.block_number - data.current.block_number;
                            if remaining > 0 {
                                // Estimate time: ~10 minutes per block
                                let minutes = remaining * 10;
                                let days = minutes / (60 * 24);
                                let hours = (minutes % (60 * 24)) / 60;

                                out.push_str(&format!(
                                    "\n*Blocks Remaining:* {}",
                                    format_number_with_commas(remaining)
                                ));

                                if days > 0 {
                                    out.push_str(&format!(
                                        "\n*Est. Time:* ~{} days {} hrs",
                                        format_number_with_commas(days),
                                        hours
                                    ));
                                } else if hours > 0 {
                                    let mins = minutes % 60;
                                    out.push_str(&format!(
                                        "\n*Est. Time:* ~{} hrs {} min",
                                        hours, mins
                                    ));
                                } else {
                                    out.push_str(&format!("\n*Est. Time:* ~{} min", minutes % 60));
                                }
                            } else {
                                out.push_str("\n*Halving:* ✅ Completed");
                            }
                        }
                    }

                    let prefix = req.prefix();
                    out.push_str(&format!(
                        "\n\n_Powered by Watcher Guru · {}markets for more_",
                        prefix
                    ));

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
                    let prefix = req.prefix();
                    respond(format!(
                        "₿ *Bitcoin (BTC)*\n\n*Price:* {}\n\n_Powered by Binance · {}markets for more_",
                        format_usd_price(price),
                        prefix
                    ));
                    return;
                }
            }
        }
    }

    respond_err("Failed to fetch Bitcoin market data.");
}
