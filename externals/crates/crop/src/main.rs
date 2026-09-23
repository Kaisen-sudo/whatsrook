use std::fs;
use std::process::Command;
use whatsrook_sdk::{Request, respond};

fn write_exif_metadata(webp_bytes: &[u8], pack_name: &str, author: &str) -> Vec<u8> {
    let json_meta = serde_json::json!({
        "sticker-pack-id": "whatsrook-sticker-pack",
        "sticker-pack-name": pack_name,
        "sticker-pack-publisher": author,
        "emojis": ["✂️"]
    });
    let json_bytes = json_meta.to_string();
    let json_bytes = json_bytes.as_bytes();

    let mut exif_payload = Vec::new();
    exif_payload.extend_from_slice(b"Exif\x00\x00II*\x00\x08\x00\x00\x00\x01\x00A\x01\x04\x00\x00\x00\x00\x00\x16\x00\x00\x00\x00\x00\x00\x00");
    exif_payload.extend_from_slice(json_bytes);

    if webp_bytes.len() < 12 || &webp_bytes[0..4] != b"RIFF" || &webp_bytes[8..12] != b"WEBP" {
        return webp_bytes.to_vec();
    }

    let mut result = Vec::new();
    result.extend_from_slice(&webp_bytes[0..12]);
    result.extend_from_slice(b"EXIF");
    let exif_len = exif_payload.len() as u32;
    result.extend_from_slice(&exif_len.to_le_bytes());
    result.extend_from_slice(&exif_payload);
    if exif_payload.len() % 2 != 0 {
        result.push(0);
    }
    result.extend_from_slice(&webp_bytes[12..]);
    let total_riff_len = (result.len() - 8) as u32;
    result[4..8].copy_from_slice(&total_riff_len.to_le_bytes());
    result
}

fn convert_to_crop_sticker(
    input_path: &str,
    pack_name: &str,
    author: &str,
) -> Result<Vec<u8>, String> {
    let tmp_dir = std::env::temp_dir();
    let out_path = format!(
        "{}/crop_{}_{}.webp",
        tmp_dir.display(),
        std::process::id(),
        rand::random::<u32>()
    );

    let status = Command::new("ffmpeg")
        .args([
            "-y", "-i", input_path,
            "-t", "8",
            "-vf", "crop='min(iw,ih)':'min(iw,ih)',scale=512:512",
            "-vcodec", "libwebp",
            "-lossless", "0",
            "-q:v", "35",
            "-compression_level", "6",
            "-loop", "0",
            "-preset", "default",
            "-an", "-pix_fmt", "yuva420p",
            &out_path,
        ])
        .status()
        .map_err(|e| format!("ffmpeg execution failed: {}", e))?;

    if !status.success() {
        let _ = fs::remove_file(&out_path);
        return Err("ffmpeg failed to process crop sticker".to_string());
    }

    let webp_raw = fs::read(&out_path)
        .map_err(|e| format!("Failed to read sticker output: {}", e))?;
    let _ = fs::remove_file(&out_path);
    Ok(write_exif_metadata(&webp_raw, pack_name, author))
}

fn main() {
    let req = Request::load();
    let mut pack_name = req.bot_name().to_string();
    let mut author = "WhatsRook".to_string();

    let query = req.query();
    if !query.is_empty() {
        let parts: Vec<&str> = query.split('|').collect();
        if !parts.is_empty() && !parts[0].trim().is_empty() {
            author = parts[0].trim().to_string();
        }
        if parts.len() > 1 && !parts[1].trim().is_empty() {
            pack_name = parts[1].trim().to_string();
        }
    }

    let cli_args: Vec<String> = std::env::args().collect();
    if cli_args.len() > 1 && !cli_args[1].starts_with('{') {
        let input_file = &cli_args[1];
        let output_file = cli_args.get(2).map(|s| s.as_str()).unwrap_or("crop.webp");
        match convert_to_crop_sticker(input_file, &pack_name, &author) {
            Ok(bytes) => {
                if let Err(e) = fs::write(output_file, &bytes) {
                    eprintln!("Error saving sticker to {}: {}", output_file, e);
                    std::process::exit(1);
                }
                println!("Saved {} bytes to {}", bytes.len(), output_file);
            }
            Err(e) => {
                eprintln!("Crop sticker error: {}", e);
                std::process::exit(1);
            }
        }
        return;
    }

    respond(format!(
        "Reply to an image or video with `{prefix}crop [author | pack]` to convert it to a square-cropped sticker.",
        prefix = req.prefix()
    ));
}
