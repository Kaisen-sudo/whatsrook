use std::collections::HashMap;
use std::fs;
use whatsrook_sdk::{respond, Request};

fn get_memory_info() -> Option<(f64, f64, f64, f64)> {
    #[cfg(target_os = "linux")]
    {
        if let Ok(content) = fs::read_to_string("/proc/meminfo") {
            let mut map: HashMap<&str, u64> = HashMap::new();
            for line in content.lines() {
                let mut parts = line.split_whitespace();
                if let (Some(key), Some(val_str)) = (parts.next(), parts.next()) {
                    let key = key.trim_end_matches(':');
                    if let Ok(val) = val_str.parse::<u64>() {
                        map.insert(key, val);
                    }
                }
            }

            if let Some(&total_kb) = map.get("MemTotal") {
                let available_kb = map.get("MemAvailable").copied().unwrap_or_else(|| {
                    let free = map.get("MemFree").copied().unwrap_or(0);
                    let buffers = map.get("Buffers").copied().unwrap_or(0);
                    let cached = map.get("Cached").copied().unwrap_or(0);
                    free + buffers + cached
                });

                let total_gb = (total_kb as f64) / (1024.0 * 1024.0);
                let available_gb = (available_kb as f64) / (1024.0 * 1024.0);
                let used_gb = total_gb - available_gb;
                let used_pct = if total_gb > 0.0 {
                    (used_gb / total_gb) * 100.0
                } else {
                    0.0
                };

                return Some((total_gb, used_gb, available_gb, used_pct));
            }
        }
    }
    None
}

fn main() {
    let _req = Request::load();

    let text = if let Some((total, used, avail, pct)) = get_memory_info() {
        format!(
            "*System Memory Information*\n\n\
             • *Total System Memory:* {:.2} GB\n\
             • *Used System Memory:* {:.2} GB ({:.1}%)\n\
             • *Available Memory:* {:.2} GB",
            total, used, pct, avail
        )
    } else {
        "*System Memory Information*\n\nMemory statistics unavailable for current platform.".to_string()
    };

    respond(text);
}
