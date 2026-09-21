use base64::Engine;
use std::fs;
use std::process::Command;
use whatsrook_sdk::{respond, send_done, send_sticker, Request};

fn write_exif_metadata(webp_bytes: &[u8], pack_name: &str, author: &str) -> Vec<u8> {
    let json_meta = serde_json::json!({
        "sticker-pack-id": "whatsrook-sticker-pack",
        "sticker-pack-name": pack_name,
        "sticker-pack-publisher": author,
        "emojis": ["🤖"]
    });
    let json_str = json_meta.to_string();
    let json_bytes = json_str.as_bytes();

    let mut exif_payload = Vec::new();
    // Standard WhatsApp WebP EXIF Header
    exif_payload.extend_from_slice(b"Exif\x00\x00II*\x00\x08\x00\x00\x00\x01\x00A\x01\x04\x00\x00\x00\x00\x00\x16\x00\x00\x00\x00\x00\x00\x00");
    exif_payload.extend_from_slice(json_bytes);

    if webp_bytes.len() < 12 || &webp_bytes[0..4] != b"RIFF" || &webp_bytes[8..12] != b"WEBP" {
        return webp_bytes.to_vec();
    }

    let mut result = Vec::new();
    result.extend_from_slice(&webp_bytes[0..12]);

    // Append EXIF chunk
    result.extend_from_slice(b"EXIF");
    let exif_len = exif_payload.len() as u32;
    result.extend_from_slice(&exif_len.to_le_bytes());
    result.extend_from_slice(&exif_payload);
    if exif_payload.len() % 2 != 0 {
        result.push(0);
    }

    // Append existing WebP payload skipping header
    result.extend_from_slice(&webp_bytes[12..]);

    // Update RIFF total size
    let total_riff_len = (result.len() - 8) as u32;
    result[4..8].copy_from_slice(&total_riff_len.to_le_bytes());

    result
}

fn convert_to_sticker(
    input_path: &str,
    is_circle: bool,
    is_crop: bool,
    pack_name: &str,
    author: &str,
) -> Result<Vec<u8>, String> {
    let tmp_dir = std::env::temp_dir();
    let out_path = format!(
        "{}/sticker_{}_{}.webp",
        tmp_dir.display(),
        std::process::id(),
        rand::random::<u32>()
    );

    let vf = if is_circle {
        "format=yuva420p,scale=512:512:force_original_aspect_ratio=decrease,pad=512:512:(ow-iw)/2:(oh-ih)/2:color=black@0,geq=alpha_expr='if(lte(hypot(X-W/2,Y-H/2),W/2),255,0)'".to_string()
    } else if is_crop {
        "crop='min(iw,ih)':'min(iw,ih)',scale=512:512".to_string()
    } else {
        "scale=512:512:force_original_aspect_ratio=decrease,pad=512:512:(ow-iw)/2:(oh-ih)/2:color=black@0".to_string()
    };

    let status = Command::new("ffmpeg")
        .args([
            "-y",
            "-i",
            input_path,
            "-t",
            "8",
            "-vf",
            &vf,
            "-vcodec",
            "libwebp",
            "-lossless",
            "0",
            "-q:v",
            "35",
            "-compression_level",
            "6",
            "-loop",
            "0",
            "-preset",
            "default",
            "-an",
            "-pix_fmt",
            "yuva420p",
            &out_path,
        ])
        .status()
        .map_err(|e| format!("ffmpeg execution failed: {}", e))?;

    if !status.success() {
        let _ = fs::remove_file(&out_path);
        return Err("ffmpeg failed to process sticker".to_string());
    }

    let webp_raw =
        fs::read(&out_path).map_err(|e| format!("Failed to read sticker output: {}", e))?;
    let _ = fs::remove_file(&out_path);

    let final_webp = write_exif_metadata(&webp_raw, pack_name, author);
    Ok(final_webp)
}

fn main() {
    let req = Request::load();
    let query = req.query();
    let cmd = req.command.to_lowercase();

    let is_circle = cmd == "circle";
    let is_crop = cmd == "crop";

    let mut pack_name = req.bot_name().to_string();
    let mut author = "WhatsRook".to_string();

    if !query.is_empty() {
        let parts: Vec<&str> = query.split('|').collect();
        if !parts.is_empty() && !parts[0].trim().is_empty() {
            author = parts[0].trim().to_string();
        }
        if parts.len() > 1 && !parts[1].trim().is_empty() {
            pack_name = parts[1].trim().to_string();
        }
    }

    // A quoted/attached media file, already downloaded locally by WhatsRook, is required
    // to actually produce a sticker. Without it we fall back to the command menu below.
    if let Some(media) = req.media() {
        if media.path.is_empty() {
            respond_missing_media(&req);
            return;
        }

        match convert_to_sticker(&media.path, is_circle, is_crop, &pack_name, &author) {
            Ok(bytes) => {
                let encoded = base64::engine::general_purpose::STANDARD.encode(&bytes);
                send_sticker(&encoded);
                send_done();
            }
            Err(e) => {
                eprintln!("Sticker generation error: {}", e);
                respond(format!("❌ Failed to generate sticker: {}", e));
            }
        }
        return;
    }

    respond_missing_media(&req);
}

/// Shows the sticker command menu (used when no media was attached to/quoted by the message).
fn respond_missing_media(req: &Request) {
    respond(format!(
        "*{bot_name} Sticker Engine*\n\n\
         • `{prefix}sticker [author | pack]` : Convert media to standard WebP sticker\n\
         • `{prefix}circle [author | pack]`  : Convert media to circular WebP sticker\n\
         • `{prefix}crop [author | pack]`    : Convert media to square cropped WebP sticker\n\
         • `{prefix}take [author | pack]`    : Re-pack metadata for any existing sticker\n\n\
         _Reply to an image, video, or sticker with one of the commands above._",
        bot_name = req.bot_name(),
        prefix = req.prefix()
    ));
}
