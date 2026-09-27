use std::fs;
use whatsrook_sdk::{respond, Request};

fn get_cpu_model() -> String {
    #[cfg(target_os = "linux")]
    {
        if let Ok(content) = fs::read_to_string("/proc/cpuinfo") {
            for line in content.lines() {
                if line.starts_with("model name")
                    || line.starts_with("Hardware")
                    || line.starts_with("Processor")
                {
                    if let Some(val) = line.split(':').nth(1) {
                        return val.trim().to_string();
                    }
                }
            }
        }
    }
    format!("{} ({})", std::env::consts::ARCH, std::env::consts::OS)
}

fn get_load_avg() -> String {
    #[cfg(target_os = "linux")]
    {
        if let Ok(content) = fs::read_to_string("/proc/loadavg") {
            let parts: Vec<&str> = content.split_whitespace().take(3).collect();
            if !parts.is_empty() {
                return parts.join(", ");
            }
        }
    }
    "N/A".to_string()
}

fn main() {
    let _req = Request::load();

    let model = get_cpu_model();
    let cores = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1);
    let load_avg = get_load_avg();
    let arch = std::env::consts::ARCH;
    let os = std::env::consts::OS;

    let text = format!(
        "*CPU Information*\n\n\
         • *Model:* {}\n\
         • *Architecture:* {}\n\
         • *OS:* {}\n\
         • *Cores/Threads:* {}\n\
         • *Load Average (1m, 5m, 15m):* {}",
        model, arch, os, cores, load_avg
    );

    respond(text);
}
