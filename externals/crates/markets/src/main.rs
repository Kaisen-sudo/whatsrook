use serde::Deserialize;
use std::collections::HashMap;
use url::form_urlencoded;
use whatsrook_sdk::{create_http_client, respond, respond_err, Request};

#[derive(Deserialize)]
struct FFInstrumentResponse {
    #[serde(default)]
    data: Vec<FFInstrumentData>,
}

#[derive(Deserialize)]
struct FFInstrumentData {
    instrument: FFInstrumentMeta,
    #[serde(default)]
    metrics: HashMap<String, FFMetric>,
    #[serde(default)]
    quotes: Vec<FFQuote>,
}

#[derive(Deserialize)]
struct FFInstrumentMeta {
    #[serde(default)]
    display_name: String,
    #[serde(default)]
    name: String,
    #[serde(default)]
    decimals: usize,
    #[serde(default)]
    is_in_holiday: bool,
}

#[derive(Deserialize)]
struct FFMetric {
    #[serde(default)]
    price: f64,
    #[serde(default)]
    high: f64,
    #[serde(default)]
    low: f64,
    #[serde(default)]
    spread: f64,
}

#[derive(Deserialize)]
struct FFQuote {
    #[serde(default)]
    bid: f64,
    #[serde(default)]
    ask: f64,
}

#[derive(Deserialize)]
struct FFBarsResponse {
    #[serde(default)]
    data: Vec<FFBarItem>,
}

#[derive(Deserialize)]
struct FFBarItem {
    #[serde(default)]
    open: f64,
    #[serde(default)]
    high: f64,
    #[serde(default)]
    low: f64,
    #[serde(default)]
    close: f64,
}

fn normalize_pair(input: &str) -> String {
    let s = input
        .trim()
        .to_uppercase()
        .replace('-', "/")
        .replace(' ', "");
    match s.as_str() {
        "GOLD" | "XAUUSD" | "GOLD/USD" => "Gold/USD".to_string(),
        "SILVER" | "XAGUSD" | "SILVER/USD" => "Silver/USD".to_string(),
        "OIL" | "WTI" | "WTI/USD" | "CRUDE" => "WTI/USD".to_string(),
        "BRENT" | "BRENT/USD" => "Brent/USD".to_string(),
        "NATGAS" | "NATGAS/USD" | "GAS" => "NatGas/USD".to_string(),
        "DOW" | "DOW/USD" | "DOWJONES" | "US30" | "DJIA" => "Dow/USD".to_string(),
        "SPX" | "SPX/USD" | "SP500" | "US500" | "S&P500" => "SPX/USD".to_string(),
        "NDX" | "NDX/USD" | "NASDAQ" | "US100" | "NAS100" => "NDX/USD".to_string(),
        "NIKKEI" | "NIKKEI225" | "NIKKEI225/USD" | "JP225" => "Nikkei225/USD".to_string(),
        "DAX" | "DAX/USD" | "GER30" | "DE30" | "GER40" => "DAX/USD".to_string(),
        "FTSE" | "FTSE100" | "FTSE100/USD" | "UK100" => "FTSE100/USD".to_string(),
        "EURUSD" => "EUR/USD".to_string(),
        "GBPUSD" => "GBP/USD".to_string(),
        "USDJPY" => "USD/JPY".to_string(),
        "USDCHF" => "USD/CHF".to_string(),
        "USDCAD" => "USD/CAD".to_string(),
        "AUDUSD" => "AUD/USD".to_string(),
        "NZDUSD" => "NZD/USD".to_string(),
        "BTCUSD" | "BTC" => "BTC/USD".to_string(),
        "ETHUSD" | "ETH" => "ETH/USD".to_string(),
        "DOGEUSD" | "DOGE" => "DOGE/USD".to_string(),
        _ => {
            if s.contains('/') {
                s
            } else if s.len() == 6 {
                format!("{}/{}", &s[0..3], &s[3..6])
            } else {
                s
            }
        }
    }
}

fn main() {
    let req = Request::load();
    let query = req.query();

    if query.is_empty() {
        respond("*Forex Factory Market Rates*\n\nUsage:\n• markets <pair> (e.g. EUR/USD, Gold/USD, BTC/USD)\n• markets all (overview of major currency & commodity pairs)");
        return;
    }

    let upper = query.trim().to_uppercase();
    if upper == "ALL" || upper == "LIST" || upper == "MENU" || upper == "OVERVIEW" {
        fetch_overview();
        return;
    }

    let pair = normalize_pair(&query);
    fetch_single_market(&pair);
}

fn fetch_overview() {
    let pairs = [
        "EUR/USD", "GBP/USD", "USD/JPY", "USD/CHF", "USD/CAD", "AUD/USD", "NZD/USD", "Gold/USD",
    ];
    let client = create_http_client(10);
    let encoded_pairs: String =
        form_urlencoded::byte_serialize(pairs.join(",").as_bytes()).collect();
    let url = format!(
        "https://mds-api.forexfactory.com/instruments?instruments={}",
        encoded_pairs
    );

    if let Ok(resp) = client.get(&url).send() {
        if resp.status().is_success() {
            if let Ok(data) = resp.json::<FFInstrumentResponse>() {
                if !data.data.is_empty() {
                    let mut out = String::from("*Forex Factory Market Overview*\n\n");
                    for item in data.data {
                        let name = if !item.instrument.display_name.is_empty() {
                            &item.instrument.display_name
                        } else {
                            &item.instrument.name
                        };
                        let mut price = 0.0;
                        if let Some(q) = item.quotes.first() {
                            price = (q.bid + q.ask) / 2.0;
                        }
                        if price == 0.0 {
                            if let Some(d1) = item.metrics.get("D1") {
                                price = d1.price;
                            }
                        }
                        let decimals = if item.instrument.decimals > 0 {
                            item.instrument.decimals
                        } else {
                            4
                        };
                        out.push_str(&format!("• *{}*: {:.*}\n", name, decimals, price));
                    }
                    respond(out.trim());
                    return;
                }
            }
        }
    }

    respond_err("Failed to fetch market rates overview.");
}

fn fetch_single_market(pair: &str) {
    let client = create_http_client(8);
    let encoded_pair: String = form_urlencoded::byte_serialize(pair.as_bytes()).collect();

    // Primary API
    let url = format!(
        "https://mds-api.forexfactory.com/instruments?instruments={}",
        encoded_pair
    );
    if let Ok(resp) = client.get(&url).send() {
        if resp.status().is_success() {
            if let Ok(data) = resp.json::<FFInstrumentResponse>() {
                if let Some(item) = data.data.first() {
                    let name = if !item.instrument.display_name.is_empty() {
                        &item.instrument.display_name
                    } else {
                        &item.instrument.name
                    };

                    let mut price = 0.0;
                    let mut high = 0.0;
                    let mut low = 0.0;
                    let mut spread = 0.0;

                    if let Some(d1) = item.metrics.get("D1") {
                        price = d1.price;
                        high = d1.high;
                        low = d1.low;
                        spread = d1.spread;
                    } else if let Some(h1) = item.metrics.get("H1") {
                        price = h1.price;
                        high = h1.high;
                        low = h1.low;
                        spread = h1.spread;
                    }

                    let (bid, ask) = if let Some(q) = item.quotes.first() {
                        if price == 0.0 {
                            price = (q.bid + q.ask) / 2.0;
                        }
                        (q.bid, q.ask)
                    } else {
                        (0.0, 0.0)
                    };

                    let decimals = if item.instrument.decimals > 0 {
                        item.instrument.decimals
                    } else {
                        4
                    };
                    let status = if item.instrument.is_in_holiday {
                        "Holiday / Closed"
                    } else {
                        "Open"
                    };

                    let mut out = format!("*Forex Factory Rates - {}*\n", name);
                    if price > 0.0 {
                        out.push_str(&format!("\n*Price:* {:.*}", decimals, price));
                    }
                    if bid > 0.0 && ask > 0.0 {
                        out.push_str(&format!(
                            "\n*Bid / Ask:* {:.*} | {:.*}",
                            decimals, bid, decimals, ask
                        ));
                    }
                    if high > 0.0 && low > 0.0 {
                        out.push_str(&format!(
                            "\n*24h High / Low:* {:.*} | {:.*}",
                            decimals, high, decimals, low
                        ));
                    }
                    if spread > 0.0 {
                        out.push_str(&format!("\n*Spread:* {:.1} pips", spread));
                    }
                    out.push_str(&format!("\n*Market Status:* {}", status));

                    respond(out);
                    return;
                }
            }
        }
    }

    // Fallback: Bars API
    let bars_url = format!(
        "https://mds-api.forexfactory.com/bars?instrument={}&interval=M5&per_page=1",
        encoded_pair
    );
    if let Ok(resp) = client.get(&bars_url).send() {
        if resp.status().is_success() {
            if let Ok(bars) = resp.json::<FFBarsResponse>() {
                if let Some(bar) = bars.data.first() {
                    let out = format!(
                        "*Forex Factory Rates - {}*\n\n*Price:* {:.2}\n*Open:* {:.2}\n*High / Low:* {:.2} | {:.2}\n*Market Status:* Active",
                        pair, bar.close, bar.open, bar.high, bar.low
                    );
                    respond(out);
                    return;
                }
            }
        }
    }

    respond(format!(
        "Could not find market rates for {:?}. Use `markets all` to view active instruments.",
        pair
    ));
}
