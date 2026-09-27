use base64::Engine;
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};
use rusttype::{Font, Scale};
use std::collections::HashMap;
use std::f64::consts::PI;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::process::{Command, Stdio};
use tiny_skia::{Color, FillRule, LineCap, Paint, PathBuilder, Pixmap, Stroke, Transform};
use whatsrook_sdk::{respond, respond_err, send_action, Action, Request};

const FONT_BYTES: &[u8] = include_bytes!("../assets/font.ttf");

const WIDTH: u32 = 720;
const HEIGHT: u32 = 720;
const FPS: u32 = 30;

const MOVE_DURATION: f64 = 1.19;
const DWELL_DURATION: f64 = 0.85;
const INTRO_DURATION: f64 = 1.2;
const NUM_TICKS: usize = 24;
const NUM_PARTICLES: usize = 32;

#[derive(Clone, Copy, Debug)]
struct DigitPos {
    angle: f64,
    radius: f64,
}

#[derive(Clone, Copy, Debug)]
struct Particle {
    angle: f64,
    dist_ratio: f64,
    size: f64,
    speed: f64,
    alpha: f64,
}

#[derive(Clone, Copy, Debug)]
struct Tick {
    angle: f64,
    inner_offset: f64,
    length: f64,
    width: f64,
    alpha: f64,
}

fn ease_in_out_cubic(t: f64) -> f64 {
    if t < 0.5 {
        4.0 * t * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
    }
}

fn ease_out_back(t: f64) -> f64 {
    let c1 = 1.70158;
    let c3 = c1 + 1.0;
    1.0 + c3 * (t - 1.0).powi(3) + c1 * (t - 1.0).powi(2)
}

fn ease_out_cubic(t: f64) -> f64 {
    1.0 - (1.0 - t).powi(3)
}

fn generate_digit_positions<R: Rng>(rng: &mut R) -> HashMap<char, DigitPos> {
    let mut digits = vec!['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];
    digits.shuffle(rng);

    let start_angle = rng.gen::<f64>() * 360.0;
    let mut cfg = HashMap::new();
    for (i, &d) in digits.iter().enumerate() {
        let sector_angle = start_angle + (i as f64) * 36.0;
        let angle_jitter = (rng.gen::<f64>() - 0.5) * 4.0;
        let angle = sector_angle + angle_jitter;
        let radius = 0.62;
        cfg.insert(d, DigitPos { angle, radius });
    }
    cfg
}

fn generate_particles<R: Rng>(rng: &mut R) -> Vec<Particle> {
    (0..NUM_PARTICLES)
        .map(|_| Particle {
            angle: rng.gen::<f64>() * PI * 2.0,
            dist_ratio: 0.2 + rng.gen::<f64>() * 0.95,
            size: 1.5 + rng.gen::<f64>() * 2.8,
            speed: (rng.gen::<f64>() - 0.5) * 0.002,
            alpha: 0.4 + rng.gen::<f64>() * 0.6,
        })
        .collect()
}

fn generate_ticks<R: Rng>(rng: &mut R) -> Vec<Tick> {
    (0..NUM_TICKS)
        .map(|i| {
            let angle =
                ((i as f64) / (NUM_TICKS as f64)) * PI * 2.0 + (rng.gen::<f64>() - 0.5) * 0.15;
            Tick {
                angle,
                inner_offset: -12.0 + (rng.gen::<f64>() - 0.5) * 16.0,
                length: 14.0 + rng.gen::<f64>() * 22.0,
                width: 1.5 + rng.gen::<f64>() * 1.5,
                alpha: 0.7 + rng.gen::<f64>() * 0.3,
            }
        })
        .collect()
}

fn get_pos(
    cfg: &HashMap<char, DigitPos>,
    digit: char,
    cx: f64,
    cy: f64,
    radius: f64,
) -> (f64, f64) {
    let c = cfg.get(&digit).copied().unwrap_or(DigitPos {
        angle: 0.0,
        radius: 0.6,
    });
    let rad = c.angle * PI / 180.0;
    let r = radius * c.radius;
    (cx + r * rad.cos(), cy + r * rad.sin())
}

#[allow(clippy::too_many_arguments)]
fn draw_dashed_line(
    pixmap: &mut Pixmap,
    x0: f64,
    y0: f64,
    x1: f64,
    y1: f64,
    dash_len: f64,
    gap_len: f64,
    alpha: f32,
) {
    let dx = x1 - x0;
    let dy = y1 - y0;
    let dist = dx.hypot(dy);
    if dist <= 0.001 {
        return;
    }
    let ux = dx / dist;
    let uy = dy / dist;

    let mut pos = 0.0;
    let mut drawing = true;

    let mut paint = Paint::default();
    paint.set_color(Color::from_rgba(1.0, 1.0, 1.0, alpha).unwrap_or(Color::WHITE));
    paint.anti_alias = true;

    let stroke = Stroke {
        width: 1.5,
        ..Default::default()
    };

    while pos < dist {
        let seg_len = if drawing { dash_len } else { gap_len };
        let end = (pos + seg_len).min(dist);
        if drawing {
            let mut pb = PathBuilder::new();
            pb.move_to((x0 + ux * pos) as f32, (y0 + uy * pos) as f32);
            pb.line_to((x0 + ux * end) as f32, (y0 + uy * end) as f32);
            if let Some(path) = pb.finish() {
                pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
            }
        }
        pos = end;
        drawing = !drawing;
    }
}

fn render_frame(
    pixmap: &mut Pixmap,
    font: &Font,
    sequence: &[char],
    cfg: &HashMap<char, DigitPos>,
    particles: &[Particle],
    ticks: &[Tick],
    now: f64,
) {
    let intro_raw = (now / INTRO_DURATION).min(1.0);
    let intro_ease = ease_out_cubic(intro_raw);
    let intro_scale = ease_out_back((intro_raw * 1.1).min(1.0));

    let float_y = (now * 1.5).sin() * 7.0 * intro_ease;
    let float_x = (now * 1.1).cos() * 4.0 * intro_ease;

    let cx = (WIDTH as f64) / 2.0 + float_x;
    let cy = (HEIGHT as f64) / 2.0 + float_y;
    let base_dial_radius = ((WIDTH.min(HEIGHT) as f64) * 0.38).max(1.0);
    let dial_radius = base_dial_radius * intro_scale.max(0.01);

    // Clear background: #0c0f17
    pixmap.fill(Color::from_rgba8(12, 15, 23, 255));

    let opening_move_start = INTRO_DURATION;
    let opening_move_end = INTRO_DURATION + MOVE_DURATION;

    let (current_x, current_y) = if sequence.is_empty() || now < opening_move_start {
        (cx, cy)
    } else if now < opening_move_end {
        let p = ease_in_out_cubic((now - opening_move_start) / MOVE_DURATION);
        let (to_x, to_y) = get_pos(cfg, sequence[0], cx, cy, dial_radius);
        (cx + (to_x - cx) * p, cy + (to_y - cy) * p)
    } else {
        let cycle_len = MOVE_DURATION + DWELL_DURATION;
        let elapsed = now - opening_move_end;
        let cycle_idx = ((elapsed / cycle_len) as usize) % sequence.len();
        let next_idx = (cycle_idx + 1) % sequence.len();
        let phase_t = elapsed % cycle_len;

        let (from_x, from_y) = get_pos(cfg, sequence[cycle_idx], cx, cy, dial_radius);
        let (to_x, to_y) = get_pos(cfg, sequence[next_idx], cx, cy, dial_radius);

        if phase_t < MOVE_DURATION {
            let p = ease_in_out_cubic(phase_t / MOVE_DURATION);
            (from_x + (to_x - from_x) * p, from_y + (to_y - from_y) * p)
        } else {
            (to_x, to_y)
        }
    };

    // Draw main dial circle
    {
        let mut pb = PathBuilder::new();
        pb.push_circle(cx as f32, cy as f32, dial_radius as f32);
        if let Some(path) = pb.finish() {
            // Fill #161618
            let mut fill_paint = Paint::default();
            fill_paint.set_color_rgba8(22, 22, 24, 255);
            fill_paint.anti_alias = true;
            pixmap.fill_path(
                &path,
                &fill_paint,
                FillRule::Winding,
                Transform::identity(),
                None,
            );

            // Stroke #ffffff
            let mut stroke_paint = Paint::default();
            let alpha = (intro_ease * 1.2).min(1.0) as f32;
            stroke_paint.set_color(Color::from_rgba(1.0, 1.0, 1.0, alpha).unwrap_or(Color::WHITE));
            stroke_paint.anti_alias = true;

            let stroke = Stroke {
                width: ((dial_radius * 0.015).max(3.0)) as f32,
                ..Default::default()
            };
            pixmap.stroke_path(&path, &stroke_paint, &stroke, Transform::identity(), None);
        }
    }

    // Draw Ticks
    for t in ticks {
        let cos = t.angle.cos();
        let sin = t.angle.sin();
        let r_inner = dial_radius + t.inner_offset;
        let r_outer = r_inner + t.length * intro_ease;

        let mut pb = PathBuilder::new();
        pb.move_to((cx + r_inner * cos) as f32, (cy + r_inner * sin) as f32);
        pb.line_to((cx + r_outer * cos) as f32, (cy + r_outer * sin) as f32);
        if let Some(path) = pb.finish() {
            let mut stroke_paint = Paint::default();
            let alpha = (t.alpha * intro_ease).clamp(0.0, 1.0) as f32;
            stroke_paint.set_color(Color::from_rgba(1.0, 1.0, 1.0, alpha).unwrap_or(Color::WHITE));
            stroke_paint.anti_alias = true;

            let stroke = Stroke {
                width: t.width as f32,
                line_cap: LineCap::Round,
                ..Default::default()
            };
            pixmap.stroke_path(&path, &stroke_paint, &stroke, Transform::identity(), None);
        }
    }

    // Draw Particles
    for p in particles {
        let angle = p.angle + p.speed * now * 1000.0;
        let pr = dial_radius * p.dist_ratio;
        let px = cx + pr * angle.cos();
        let py = cy + pr * angle.sin();

        let mut pb = PathBuilder::new();
        pb.push_circle(px as f32, py as f32, (p.size * intro_ease).max(0.1) as f32);
        if let Some(path) = pb.finish() {
            let mut fill_paint = Paint::default();
            let alpha = (p.alpha * intro_ease).clamp(0.0, 1.0) as f32;
            fill_paint.set_color(Color::from_rgba(1.0, 1.0, 1.0, alpha).unwrap_or(Color::WHITE));
            fill_paint.anti_alias = true;
            pixmap.fill_path(
                &path,
                &fill_paint,
                FillRule::Winding,
                Transform::identity(),
                None,
            );
        }
    }

    // Draw Dashed Line to Indicator Badge
    draw_dashed_line(
        pixmap,
        cx,
        cy,
        current_x,
        current_y,
        4.0,
        6.0,
        (0.08 * intro_ease) as f32,
    );

    // Draw Badge
    let badge_radius = dial_radius * 0.14;
    // Outer Glow
    {
        let mut pb = PathBuilder::new();
        pb.push_circle(
            current_x as f32,
            current_y as f32,
            ((badge_radius + 2.0) * intro_ease).max(1.0) as f32,
        );
        if let Some(path) = pb.finish() {
            let mut fill_paint = Paint::default();
            let alpha = (0.4 * intro_ease).clamp(0.0, 1.0) as f32;
            fill_paint.set_color(
                Color::from_rgba(43.0 / 255.0, 91.0 / 255.0, 102.0 / 255.0, alpha)
                    .unwrap_or(Color::BLACK),
            );
            fill_paint.anti_alias = true;
            pixmap.fill_path(
                &path,
                &fill_paint,
                FillRule::Winding,
                Transform::identity(),
                None,
            );
        }
    }
    // Badge Main
    {
        let mut pb = PathBuilder::new();
        pb.push_circle(
            current_x as f32,
            current_y as f32,
            (badge_radius * intro_ease).max(1.0) as f32,
        );
        if let Some(path) = pb.finish() {
            let mut fill_paint = Paint::default();
            fill_paint.set_color_rgba8(43, 91, 102, (255.0 * intro_ease.clamp(0.0, 1.0)) as u8);
            fill_paint.anti_alias = true;
            pixmap.fill_path(
                &path,
                &fill_paint,
                FillRule::Winding,
                Transform::identity(),
                None,
            );

            let mut stroke_paint = Paint::default();
            stroke_paint.set_color(
                Color::from_rgba(1.0, 1.0, 1.0, intro_ease.clamp(0.0, 1.0) as f32)
                    .unwrap_or(Color::WHITE),
            );
            stroke_paint.anti_alias = true;

            let stroke = Stroke {
                width: 2.2,
                ..Default::default()
            };
            pixmap.stroke_path(&path, &stroke_paint, &stroke, Transform::identity(), None);
        }
    }

    // Draw Digits
    let font_size = (base_dial_radius * 0.38) as f32;
    let scale = Scale::uniform(font_size);
    let v_metrics = font.v_metrics(scale);

    for &digit in cfg.keys() {
        let (x, y) = get_pos(cfg, digit, cx, cy, dial_radius);
        let glyph = font.glyph(digit).scaled(scale);
        let h_metrics = glyph.h_metrics();
        let glyph_pos = glyph.positioned(rusttype::point(
            (x - (h_metrics.advance_width as f64) / 2.0) as f32,
            (y + ((v_metrics.ascent - v_metrics.descent) as f64) / 2.0
                - (v_metrics.ascent as f64) * 0.08) as f32,
        ));

        if let Some(bb) = glyph_pos.pixel_bounding_box() {
            let pixels = pixmap.pixels_mut();
            glyph_pos.draw(|gx, gy, v| {
                let px = bb.min.x + gx as i32;
                let py = bb.min.y + gy as i32;
                if px >= 0 && px < (WIDTH as i32) && py >= 0 && py < (HEIGHT as i32) {
                    let idx = (py as usize) * (WIDTH as usize) + (px as usize);
                    let pixel = &mut pixels[idx];
                    let alpha = (v * (intro_ease as f32)).clamp(0.0, 1.0);
                    if alpha > 0.0 {
                        let src_r = 255.0 * alpha;
                        let src_g = 255.0 * alpha;
                        let src_b = 255.0 * alpha;
                        let inv_a = 1.0 - alpha;
                        let cur_r = pixel.red() as f32;
                        let cur_g = pixel.green() as f32;
                        let cur_b = pixel.blue() as f32;
                        let new_r = (cur_r * inv_a + src_r).clamp(0.0, 255.0) as u8;
                        let new_g = (cur_g * inv_a + src_g).clamp(0.0, 255.0) as u8;
                        let new_b = (cur_b * inv_a + src_b).clamp(0.0, 255.0) as u8;
                        *pixel =
                            tiny_skia::PremultipliedColorU8::from_rgba(new_r, new_g, new_b, 255)
                                .unwrap();
                    }
                }
            });
        }
    }
}

pub fn generate_video(code: &str, seconds: f64, seed: Option<u64>) -> Result<Vec<u8>, String> {
    if code.is_empty() {
        return Err("Captcha code must not be empty".to_string());
    }
    for c in code.chars() {
        if !c.is_ascii_digit() {
            return Err(format!("Captcha code must be numeric, got {:?}", code));
        }
    }

    let font = Font::try_from_bytes(FONT_BYTES).ok_or("Failed to load embedded font")?;
    let sequence: Vec<char> = code.chars().collect();

    let mut rng = match seed {
        Some(s) => rand::rngs::StdRng::seed_from_u64(s),
        None => rand::rngs::StdRng::from_entropy(),
    };

    let cfg = generate_digit_positions(&mut rng);
    let particles = generate_particles(&mut rng);
    let ticks = generate_ticks(&mut rng);

    let duration = if seconds <= 0.0 { 8.0 } else { seconds };
    let total_frames = (duration * (FPS as f64)).round() as u32;

    let tmp_path = format!(
        "{}/captcha_{}_{}.mp4",
        std::env::temp_dir().display(),
        std::process::id(),
        rand::random::<u32>()
    );

    let mut ffmpeg = Command::new("ffmpeg")
        .args([
            "-y",
            "-f",
            "rawvideo",
            "-pix_fmt",
            "rgba",
            "-s",
            &format!("{}x{}", WIDTH, HEIGHT),
            "-framerate",
            &FPS.to_string(),
            "-i",
            "-",
            "-c:v",
            "libx264",
            "-preset",
            "ultrafast",
            "-pix_fmt",
            "yuv420p",
            "-crf",
            "23",
            "-movflags",
            "+faststart",
            &tmp_path,
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("Failed to spawn ffmpeg (ensure ffmpeg is on PATH): {}", e))?;

    let mut stdin = ffmpeg.stdin.take().ok_or("Failed to open ffmpeg stdin")?;

    let mut pixmap = Pixmap::new(WIDTH, HEIGHT).ok_or("Failed to allocate Pixmap")?;

    for i in 0..total_frames {
        let t = (i as f64) / (FPS as f64);
        render_frame(&mut pixmap, &font, &sequence, &cfg, &particles, &ticks, t);
        if let Err(e) = stdin.write_all(pixmap.data()) {
            let _ = ffmpeg.kill();
            let _ = fs::remove_file(&tmp_path);
            return Err(format!("Failed writing frame {} to ffmpeg: {}", i, e));
        }
    }
    drop(stdin);

    let status = ffmpeg
        .wait()
        .map_err(|e| format!("Error waiting for ffmpeg: {}", e))?;
    if !status.success() {
        let _ = fs::remove_file(&tmp_path);
        return Err(format!("ffmpeg exited with non-zero status: {:?}", status));
    }

    let mut file =
        File::open(&tmp_path).map_err(|e| format!("Failed to open generated video: {}", e))?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)
        .map_err(|e| format!("Failed to read video file: {}", e))?;
    let _ = fs::remove_file(&tmp_path);

    Ok(buffer)
}

fn main() {
    let req = Request::load();
    let query = req.query();

    // Check if directly called as CLI (e.g. `captcha 3681` or `captcha 3681 output.mp4`)
    let cli_args: Vec<String> = std::env::args().collect();
    if cli_args.len() > 1 && !cli_args[1].starts_with('{') {
        let first_arg = cli_args[1].trim();
        if first_arg == "--help" || first_arg == "-h" {
            println!("Usage: captcha [CODE] [OUTPUT_FILE]");
            println!("Example: captcha 3681 out.mp4");
            return;
        }

        // Direct code generation to file
        let code = if first_arg.chars().all(|c| c.is_ascii_digit()) && !first_arg.is_empty() {
            first_arg.to_string()
        } else {
            let mut rng = rand::thread_rng();
            format!("{:04}", rng.gen_range(0..10000))
        };

        let output_file = cli_args.get(2).map(|s| s.as_str()).unwrap_or("captcha.mp4");
        println!(
            "Generating captcha video for code {} -> {}...",
            code, output_file
        );
        match generate_video(&code, 8.0, None) {
            Ok(bytes) => {
                if let Err(e) = fs::write(output_file, &bytes) {
                    eprintln!("Error saving video to {}: {}", output_file, e);
                    std::process::exit(1);
                }
                println!("Saved {} bytes to {}", bytes.len(), output_file);
                return;
            }
            Err(err) => {
                eprintln!("Video generation failed: {}", err);
                std::process::exit(1);
            }
        }
    }

    // WhatsRook Plugin Protocol Handler
    if query.is_empty() || query.eq_ignore_ascii_case("help") {
        respond(format!(
            "*{bot_name} Captcha Generator*\n\n\
             • `{prefix}captcha <code>` : Generate an animated verification code video (e.g. `{prefix}captcha 3681`)\n\
             • `{prefix}captcha test`   : Generate a random 4-digit verification video demo",
            bot_name = req.bot_name(),
            prefix = req.prefix()
        ));
        return;
    }

    let code = if query.eq_ignore_ascii_case("test") || query.eq_ignore_ascii_case("demo") {
        let mut rng = rand::thread_rng();
        format!("{:04}", rng.gen_range(0..10000))
    } else {
        let clean = query.trim();
        if clean.len() < 3 || clean.len() > 8 || !clean.chars().all(|c| c.is_ascii_digit()) {
            respond_err("Invalid captcha code. Please provide a numeric code between 3 and 8 digits (e.g. `3681`).");
        }
        clean.to_string()
    };

    match generate_video(&code, 8.0, None) {
        Ok(bytes) => {
            let b64 = format!(
                "data:video/mp4;base64,{}",
                base64::engine::general_purpose::STANDARD.encode(&bytes)
            );
            let caption = format!("🎬 *Verification Captcha*\nCode: `{}`", code);
            send_action(&Action::SendVideo {
                data: &b64,
                caption: Some(&caption),
                mimetype: Some("video/mp4"),
                gif_playback: true,
            });
        }
        Err(err) => {
            respond_err(format!("Failed to generate captcha video: {}", err));
        }
    }
}
