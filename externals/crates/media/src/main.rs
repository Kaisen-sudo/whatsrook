use std::fs;
use std::process::Command;
use whatsrook_sdk::{respond, Request};

fn convert_to_mp4(input_path: &str) -> Result<Vec<u8>, String> {
    let tmp_dir = std::env::temp_dir();
    let out_path = format!(
        "{}/media_out_{}_{}.mp4",
        tmp_dir.display(),
        std::process::id(),
        rand::random::<u32>()
    );

    let status = Command::new("ffmpeg")
        .args([
            "-y",
            "-i",
            input_path,
            "-c:v",
            "libx264",
            "-pix_fmt",
            "yuv420p",
            "-preset",
            "ultrafast",
            "-movflags",
            "+faststart",
            &out_path,
        ])
        .status()
        .map_err(|e| format!("ffmpeg execution failed: {}", e))?;

    if !status.success() {
        let _ = fs::remove_file(&out_path);
        return Err("ffmpeg MP4 conversion failed".to_string());
    }

    let bytes = fs::read(&out_path).map_err(|e| format!("Failed to read MP4 file: {}", e))?;
    let _ = fs::remove_file(&out_path);
    Ok(bytes)
}

fn convert_to_mp3(input_path: &str) -> Result<Vec<u8>, String> {
    let tmp_dir = std::env::temp_dir();
    let out_path = format!(
        "{}/media_out_{}_{}.mp3",
        tmp_dir.display(),
        std::process::id(),
        rand::random::<u32>()
    );

    let status = Command::new("ffmpeg")
        .args([
            "-y", "-i", input_path, "-vn", "-ar", "16000", "-ac", "1", "-b:a", "64k", &out_path,
        ])
        .status()
        .map_err(|e| format!("ffmpeg execution failed: {}", e))?;

    if !status.success() {
        let _ = fs::remove_file(&out_path);
        return Err("ffmpeg MP3 conversion failed".to_string());
    }

    let bytes = fs::read(&out_path).map_err(|e| format!("Failed to read MP3 file: {}", e))?;
    let _ = fs::remove_file(&out_path);
    Ok(bytes)
}

fn create_black_video(input_path: &str) -> Result<Vec<u8>, String> {
    let tmp_dir = std::env::temp_dir();
    let out_path = format!(
        "{}/media_black_{}_{}.mp4",
        tmp_dir.display(),
        std::process::id(),
        rand::random::<u32>()
    );

    let status = Command::new("ffmpeg")
        .args([
            "-y",
            "-f",
            "lavfi",
            "-i",
            "color=c=black:s=720x720:r=30",
            "-i",
            input_path,
            "-c:v",
            "libx264",
            "-tune",
            "stillimage",
            "-c:a",
            "aac",
            "-b:a",
            "128k",
            "-pix_fmt",
            "yuv420p",
            "-shortest",
            &out_path,
        ])
        .status()
        .map_err(|e| format!("ffmpeg execution failed: {}", e))?;

    if !status.success() {
        let _ = fs::remove_file(&out_path);
        return Err("ffmpeg black video generation failed".to_string());
    }

    let bytes = fs::read(&out_path).map_err(|e| format!("Failed to read video output: {}", e))?;
    let _ = fs::remove_file(&out_path);
    Ok(bytes)
}

fn main() {
    let req = Request::load();
    let cmd = req.command.to_lowercase();

    let cli_args: Vec<String> = std::env::args().collect();
    if cli_args.len() > 1 && !cli_args[1].starts_with('{') {
        let input_file = &cli_args[1];
        let output_file = cli_args.get(2).map(|s| s.as_str()).unwrap_or("output.mp4");

        let result = if cmd == "mp3" || output_file.ends_with(".mp3") {
            convert_to_mp3(input_file)
        } else if cmd == "black" {
            create_black_video(input_file)
        } else {
            convert_to_mp4(input_file)
        };

        match result {
            Ok(bytes) => {
                if let Err(e) = fs::write(output_file, &bytes) {
                    eprintln!("Error saving to {}: {}", output_file, e);
                    std::process::exit(1);
                }
                println!("Saved {} bytes to {}", bytes.len(), output_file);
                return;
            }
            Err(e) => {
                eprintln!("Media processing error: {}", e);
                std::process::exit(1);
            }
        }
    }

    respond(format!(
        "*{bot_name} Media Engine*\n\n\
         • `{prefix}mp4`   : Convert quoted sticker/audio/video to MP4 format\n\
         • `{prefix}mp3`   : Convert quoted video/audio to MP3 audio format\n\
         • `{prefix}black` : Create a black background video with attached audio track\n\
         • `{prefix}trim <start> <end>` : Trim a video to target duration",
        bot_name = req.bot_name(),
        prefix = req.prefix()
    ));
}
