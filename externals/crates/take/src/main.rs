use std::fs;
use whatsrook_sdk::{respond, respond_err, Request};

/// Replaces the EXIF sticker metadata chunk in a WebP file.
fn rewrite_exif_metadata(webp_bytes: &[u8], pack_name: &str, author: &str) -> Vec<u8> {
    let json_meta = serde_json::json!({
        "sticker-pack-id": "whatsrook-sticker-pack",
        "sticker-pack-name": pack_name,
        "sticker-pack-publisher": author,
        "emojis": ["🤖"]
    });
    let json_bytes = json_meta.to_string();
    let json_bytes = json_bytes.as_bytes();

    let mut exif_payload = Vec::new();
    exif_payload.extend_from_slice(b"Exif\x00\x00II*\x00\x08\x00\x00\x00\x01\x00A\x01\x04\x00\x00\x00\x00\x00\x16\x00\x00\x00\x00\x00\x00\x00");
    exif_payload.extend_from_slice(json_bytes);

    if webp_bytes.len() < 12 || &webp_bytes[0..4] != b"RIFF" || &webp_bytes[8..12] != b"WEBP" {
        // Not a valid WebP — return as-is.
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
    // Skip any existing EXIF chunk in the original before appending the rest.
    let mut offset = 12usize;
    while offset + 8 <= webp_bytes.len() {
        let chunk_id = &webp_bytes[offset..offset + 4];
        let chunk_len = u32::from_le_bytes([
            webp_bytes[offset + 4],
            webp_bytes[offset + 5],
            webp_bytes[offset + 6],
            webp_bytes[offset + 7],
        ]) as usize;
        let padded_len = chunk_len + (chunk_len % 2);
        if chunk_id == b"EXIF" {
            // Skip the existing EXIF chunk.
            offset += 8 + padded_len;
            continue;
        }
        let end = (offset + 8 + padded_len).min(webp_bytes.len());
        result.extend_from_slice(&webp_bytes[offset..end]);
        offset += 8 + padded_len;
    }
    let total_riff_len = (result.len() - 8) as u32;
    result[4..8].copy_from_slice(&total_riff_len.to_le_bytes());
    result
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

    // WhatsRook passes the quoted sticker's local file path as argv[1].
    let cli_args: Vec<String> = std::env::args().collect();
    if cli_args.len() > 1 && !cli_args[1].starts_with('{') {
        let input_file = &cli_args[1];
        let output_file = cli_args.get(2).map(|s| s.as_str()).unwrap_or("take.webp");

        let webp_bytes = fs::read(input_file)
            .unwrap_or_else(|e| respond_err(format!("Failed to read sticker file: {}", e)));

        let repacked = rewrite_exif_metadata(&webp_bytes, &pack_name, &author);

        if let Err(e) = fs::write(output_file, &repacked) {
            eprintln!("Error saving repacked sticker to {}: {}", output_file, e);
            std::process::exit(1);
        }
        println!("Saved {} bytes to {}", repacked.len(), output_file);
        return;
    }

    respond(format!(
        "Reply to a sticker with `{prefix}take [author | pack]` to re-pack its metadata.",
        prefix = req.prefix()
    ));
}
