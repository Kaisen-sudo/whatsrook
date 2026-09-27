// Package dl implements an omni-media downloader plugin utilizing the yt-dlp
// execution environment, ffmpeg media transcoding pipelines, and interactive
// WhatsApp poll routing.
package dl

import (
	"context"
	"encoding/json"
	"fmt"
	"net/url"
	"os"
	"os/exec"
	"path/filepath"
	"regexp"
	"strings"
	"time"

	"go.mau.fi/whatsmeow"
	"go.mau.fi/whatsmeow/proto/waE2E"
	"google.golang.org/protobuf/proto"

	"whatsrook/cmd/dispatch"
	"whatsrook/util/builder"
	"whatsrook/util/logger"
	"whatsrook/util/media"
)

func init() {
	dispatch.Register(&dispatch.Command{
		Name:        "dl",
		Alias:       "download,ytdl,ytdlp",
		Description: "Download video, audio, or image media from any URL with interactive format selection",
		Category:    "tools",
		IsPublic:    true,
		Handler:     handleDL,
	})
}

// MediaMeta represents structured metadata extracted by yt-dlp.
type MediaMeta struct {
	ID           string      `json:"id"`
	Title        string      `json:"title"`
	Description  string      `json:"description"`
	Extractor    string      `json:"extractor"`
	ExtractorKey string      `json:"extractor_key"`
	Duration     float64     `json:"duration"`
	Thumbnail    string      `json:"thumbnail"`
	Ext          string      `json:"ext"`
	URL          string      `json:"url"`
	VCodec       string      `json:"vcodec"`
	ACodec       string      `json:"acodec"`
	Type         string      `json:"_type"`
	Entries      []MediaMeta `json:"entries"`
}

// IsImage returns true if the extracted metadata indicates an image asset.
func (m *MediaMeta) IsImage() bool {
	if strings.EqualFold(m.Type, "image") {
		return true
	}
	cleanExt := strings.ToLower(strings.TrimPrefix(m.Ext, "."))
	switch cleanExt {
	case "jpg", "jpeg", "png", "webp", "gif", "bmp", "tiff", "heic", "avif":
		return true
	}
	if (m.VCodec == "none" || m.VCodec == "") && (m.ACodec == "none" || m.ACodec == "") && m.Duration == 0 {
		if cleanExt != "" {
			return true
		}
	}
	if len(m.Entries) > 0 {
		return m.Entries[0].IsImage()
	}
	return false
}

// GetTitle returns a safe title fallback.
func (m *MediaMeta) GetTitle() string {
	t := strings.TrimSpace(m.Title)
	if t != "" {
		return t
	}
	if len(m.Entries) > 0 {
		if sub := strings.TrimSpace(m.Entries[0].Title); sub != "" {
			return sub
		}
	}
	if m.ID != "" {
		return m.ID
	}
	return "Media Download"
}

var urlRegex = regexp.MustCompile(`(?i)\bhttps?://[^\s<>"]+`)

// extractTargetURL extracts the media URL from command arguments or quoted message text.
func extractTargetURL(ctx *dispatch.Context) (string, []string) {
	// 1. Check in command arguments
	var explicitArgs []string
	var targetURL string
	for _, arg := range ctx.Args {
		if targetURL == "" && (strings.HasPrefix(arg, "http://") || strings.HasPrefix(arg, "https://")) {
			targetURL = arg
		} else {
			explicitArgs = append(explicitArgs, arg)
		}
	}

	if targetURL != "" {
		return cleanURL(targetURL), explicitArgs
	}

	// 2. Check full raw arguments string
	if match := urlRegex.FindString(ctx.RawArgs); match != "" {
		return cleanURL(match), explicitArgs
	}

	// 3. Check quoted message body / caption
	if quoted := ctx.GetQuotedMessage(); quoted != nil {
		if text := quoted.GetConversation(); text != "" {
			if match := urlRegex.FindString(text); match != "" {
				return cleanURL(match), explicitArgs
			}
		}
		if ext := quoted.GetExtendedTextMessage(); ext != nil && ext.GetText() != "" {
			if match := urlRegex.FindString(ext.GetText()); match != "" {
				return cleanURL(match), explicitArgs
			}
		}
		if img := quoted.GetImageMessage(); img != nil && img.GetCaption() != "" {
			if match := urlRegex.FindString(img.GetCaption()); match != "" {
				return cleanURL(match), explicitArgs
			}
		}
		if vid := quoted.GetVideoMessage(); vid != nil && vid.GetCaption() != "" {
			if match := urlRegex.FindString(vid.GetCaption()); match != "" {
				return cleanURL(match), explicitArgs
			}
		}
		if doc := quoted.GetDocumentMessage(); doc != nil && doc.GetCaption() != "" {
			if match := urlRegex.FindString(doc.GetCaption()); match != "" {
				return cleanURL(match), explicitArgs
			}
		}
	}

	return "", explicitArgs
}

func cleanURL(raw string) string {
	raw = strings.TrimSpace(raw)
	raw = strings.TrimRight(raw, ".,;!?>)]}")
	return raw
}

func handleDL(ctx *dispatch.Context) error {
	// Handle cookie subcommands: .dl cookie ...
	if len(ctx.Args) > 0 {
		first := strings.ToLower(ctx.Args[0])
		if first == "cookie" || first == "cookies" {
			return handleCookieCommand(ctx)
		}
	}

	targetURL, remainingArgs := extractTargetURL(ctx)
	if targetURL == "" {
		return sendUsage(ctx)
	}

	// Verify required external tools
	if _, err := exec.LookPath("yt-dlp"); err != nil {
		return ctx.Reply("yt-dlp executable was not found in system PATH. Please ensure yt-dlp is installed.")
	}
	if _, err := exec.LookPath("ffmpeg"); err != nil {
		return ctx.Reply("ffmpeg executable was not found in system PATH. Please ensure ffmpeg is installed.")
	}

	// 1. If direct image URL, download and send directly without yt-dlp overhead
	if isImageURL(targetURL) {
		logger.Info("handleDL: direct image URL detected", "url", targetURL)
		if err := downloadAndSendDirectImage(ctx, targetURL); err == nil {
			return nil
		}
		logger.Warn("handleDL: direct image fetch failed, falling back to yt-dlp", "url", targetURL)
	}

	// 2. Fetch metadata using yt-dlp -J
	meta, errMeta := fetchMetadata(ctx, targetURL)
	if errMeta != nil {
		logger.Warn("handleDL: yt-dlp metadata fetch failed, attempting social scraper fallback", "url", targetURL, "err", errMeta)

		// Attempt headless browser scraper fallback via Bun & Puppeteer
		if scraped, errScrape := runSocialScraper(ctx, targetURL); errScrape == nil && len(scraped.Media) > 0 {
			logger.Info("handleDL: social scraper found media", "url", targetURL, "platform", scraped.Platform, "count", len(scraped.Media))
			if errDeliver := deliverScrapedMedia(ctx, scraped); errDeliver == nil {
				return nil
			} else {
				logger.Error("handleDL: failed delivering scraped media", "err", errDeliver)
			}
		}

		return sendFailureWithCookiePrompt(ctx, errMeta)
	}

	// 3. Automatically detect if the downloadable is an image (single or multi-image)
	if meta.IsImage() {
		logger.Info("handleDL: target identified as image", "url", targetURL, "ext", meta.Ext, "entries", len(meta.Entries))
		if err := downloadAndSendImages(ctx, targetURL, meta); err != nil {
			// If yt-dlp failed downloading images, attempt social scraper fallback
			if scraped, errScrape := runSocialScraper(ctx, targetURL); errScrape == nil && len(scraped.Media) > 0 {
				if errDeliver := deliverScrapedMedia(ctx, scraped); errDeliver == nil {
					return nil
				}
			}
			return sendFailureWithCookiePrompt(ctx, err)
		}
		return nil
	}

	// 3. Check for explicit format selection passed via argument: .dl <url> audio / .dl <url> video
	formatChoice := ""
	for _, arg := range remainingArgs {
		clean := strings.ToLower(strings.TrimSpace(arg))
		if clean == "audio" || clean == "mp3" || clean == "opus" || clean == "sound" || clean == "-a" {
			formatChoice = "audio"
			break
		}
		if clean == "video" || clean == "mp4" || clean == "-v" {
			formatChoice = "video"
			break
		}
	}

	if formatChoice == "audio" {
		if err := downloadAndSendAudio(ctx, targetURL, meta); err != nil {
			return sendFailureWithCookiePrompt(ctx, err)
		}
		return nil
	}
	if formatChoice == "video" {
		if err := downloadAndSendVideo(ctx, targetURL, meta); err != nil {
			return sendFailureWithCookiePrompt(ctx, err)
		}
		return nil
	}

	// 4. Send interactive WhatsApp poll: Video / Audio
	qTitle := meta.GetTitle()
	if len(qTitle) > 100 {
		qTitle = qTitle[:97] + "..."
	}
	durStr := formatDurationSeconds(meta.Duration)

	var question string
	if durStr != "" {
		question = fmt.Sprintf("Download: %s\nDuration: %s\nChoose media format:", qTitle, durStr)
	} else {
		question = fmt.Sprintf("Download: %s\nChoose media format:", qTitle)
	}
	if len(question) > 240 {
		question = question[:240]
	}

	poll := ctx.Poll(question)
	poll.AddOption("Video")
	poll.AddOption("Audio")
	poll.AllowedSenders(ctx.Sender)
	poll.AutoDelete(true)

	err := poll.OnceReply(func(req builder.PollRequest, res *builder.Response) {
		selected := ""
		if len(req.SelectedOptions) > 0 {
			selected = req.SelectedOptions[0]
		}
		selectedLower := strings.ToLower(selected)

		if strings.Contains(selectedLower, "audio") {
			if err := downloadAndSendAudio(ctx, targetURL, meta); err != nil {
				sendFailureWithCookiePrompt(ctx, err)
			}
		} else {
			if err := downloadAndSendVideo(ctx, targetURL, meta); err != nil {
				sendFailureWithCookiePrompt(ctx, err)
			}
		}
	})

	if err != nil {
		logger.Error("handleDL: failed to send interactive poll, falling back to direct video", "err", err)
		return downloadAndSendVideo(ctx, targetURL, meta)
	}

	return nil
}

// fetchMetadata runs yt-dlp -J to retrieve JSON metadata for the media item.
func fetchMetadata(ctx *dispatch.Context, rawURL string) (*MediaMeta, error) {
	cookiesPath, cleanup := getCookiesFilePath(ctx, rawURL)
	defer cleanup()

	args := []string{"-J", "--no-warnings", "--no-playlist"}
	if cookiesPath != "" {
		args = append(args, "--cookies", cookiesPath)
	}
	args = append(args, rawURL)

	timeoutCtx, cancel := context.WithTimeout(ctx.GetSendContext(), 45*time.Second)
	defer cancel()

	cmd := exec.CommandContext(timeoutCtx, "yt-dlp", args...)
	out, err := cmd.CombinedOutput()
	if err != nil {
		return nil, fmt.Errorf("yt-dlp metadata dump error: %w (%s)", err, strings.TrimSpace(string(out)))
	}

	var meta MediaMeta
	if err := json.Unmarshal(out, &meta); err != nil {
		return nil, fmt.Errorf("unmarshal yt-dlp metadata failed: %w", err)
	}

	return &meta, nil
}

// downloadAndSendAudio downloads the best audio track, transcodes it to Opus OGG, and sends it.
func downloadAndSendAudio(ctx *dispatch.Context, rawURL string, meta *MediaMeta) error {
	cookiesPath, cleanup := getCookiesFilePath(ctx, rawURL)
	defer cleanup()

	nowNano := time.Now().UnixNano()
	rawAudioTpl := filepath.Join(os.TempDir(), fmt.Sprintf("ytdl_aud_raw_%d.%%(ext)s", nowNano))
	opusOut := filepath.Join(os.TempDir(), fmt.Sprintf("ytdl_aud_out_%d.opus", nowNano))

	defer func() {
		if matches, _ := filepath.Glob(filepath.Join(os.TempDir(), fmt.Sprintf("ytdl_aud_raw_%d.*", nowNano))); len(matches) > 0 {
			for _, m := range matches {
				_ = os.Remove(m)
			}
		}
		_ = os.Remove(opusOut)
	}()

	dlCtx, cancel := context.WithTimeout(ctx.GetSendContext(), 5*time.Minute)
	defer cancel()

	args := []string{
		"-f", "bestaudio/best",
		"--no-warnings",
		"--no-playlist",
		"--max-filesize", "90M",
		"-o", rawAudioTpl,
	}
	if cookiesPath != "" {
		args = append(args, "--cookies", cookiesPath)
	}
	args = append(args, rawURL)

	cmd := exec.CommandContext(dlCtx, "yt-dlp", args...)
	if out, err := cmd.CombinedOutput(); err != nil {
		return fmt.Errorf("audio download failed: %w (%s)", err, strings.TrimSpace(string(out)))
	}

	matches, _ := filepath.Glob(filepath.Join(os.TempDir(), fmt.Sprintf("ytdl_aud_raw_%d.*", nowNano)))
	if len(matches) == 0 {
		return fmt.Errorf("downloaded audio file not found on disk")
	}
	rawAudioFile := matches[0]

	// Transcode audio to WhatsApp-compatible Opus OGG (48kHz, mono, VoIP tuned)
	if err := EnsureWhatsAppOpus(ctx.GetSendContext(), rawAudioFile, opusOut); err != nil {
		logger.Warn("EnsureWhatsAppOpus transcode failed, attempting OpusPTTConvert on raw bytes", "err", err)
		rawBytes, errRead := os.ReadFile(rawAudioFile)
		if errRead != nil {
			return fmt.Errorf("read raw audio file failed: %w", errRead)
		}
		return sendOpusAudioPayload(ctx, rawBytes)
	}

	opusBytes, err := os.ReadFile(opusOut)
	if err != nil || len(opusBytes) == 0 {
		return fmt.Errorf("failed to read transcoded opus file: %w", err)
	}

	return sendOpusAudioPayload(ctx, opusBytes)
}

// downloadAndSendVideo downloads video, converts it to guarantee WhatsApp compatibility, and sends with caption.
func downloadAndSendVideo(ctx *dispatch.Context, rawURL string, meta *MediaMeta) error {
	cookiesPath, cleanup := getCookiesFilePath(ctx, rawURL)
	defer cleanup()

	nowNano := time.Now().UnixNano()
	rawVideoTpl := filepath.Join(os.TempDir(), fmt.Sprintf("ytdl_vid_raw_%d.%%(ext)s", nowNano))
	waVideoOut := filepath.Join(os.TempDir(), fmt.Sprintf("ytdl_vid_wa_%d.mp4", nowNano))

	defer func() {
		if matches, _ := filepath.Glob(filepath.Join(os.TempDir(), fmt.Sprintf("ytdl_vid_raw_%d.*", nowNano))); len(matches) > 0 {
			for _, m := range matches {
				_ = os.Remove(m)
			}
		}
		_ = os.Remove(waVideoOut)
	}()

	dlCtx, cancel := context.WithTimeout(ctx.GetSendContext(), 7*time.Minute)
	defer cancel()

	args := []string{
		"-f", "bestvideo[ext=mp4]+bestaudio[ext=m4a]/bestvideo+bestaudio/best[ext=mp4]/best",
		"--no-warnings",
		"--no-playlist",
		"--max-filesize", "90M",
		"-o", rawVideoTpl,
	}
	if cookiesPath != "" {
		args = append(args, "--cookies", cookiesPath)
	}
	args = append(args, rawURL)

	cmd := exec.CommandContext(dlCtx, "yt-dlp", args...)
	if out, err := cmd.CombinedOutput(); err != nil {
		return fmt.Errorf("video download failed: %w (%s)", err, strings.TrimSpace(string(out)))
	}

	matches, _ := filepath.Glob(filepath.Join(os.TempDir(), fmt.Sprintf("ytdl_vid_raw_%d.*", nowNano)))
	if len(matches) == 0 {
		return fmt.Errorf("downloaded video file not found on disk")
	}
	rawVideoFile := matches[0]

	// Transcode / remux video to ensure WhatsApp H.264/AAC/yuv420p/+faststart compatibility
	transcodeCtx, cancelTranscode := context.WithTimeout(ctx.GetSendContext(), 4*time.Minute)
	defer cancelTranscode()

	transcodeErr := EnsureWhatsAppVideo(transcodeCtx, rawVideoFile, waVideoOut)
	targetVideoPath := waVideoOut
	if transcodeErr != nil {
		logger.Warn("EnsureWhatsAppVideo failed, falling back to original video file", "err", transcodeErr)
		targetVideoPath = rawVideoFile
	}

	videoBytes, err := os.ReadFile(targetVideoPath)
	if err != nil || len(videoBytes) == 0 {
		return fmt.Errorf("failed to read processed video file: %w", err)
	}

	caption := buildCaption(meta)
	return ctx.ReplyWithVideo(videoBytes, "video/mp4", caption)
}

// EnsureWhatsAppVideo converts or remuxes a video file to meet WhatsApp's strict compatibility requirements:
// - MP4 container with faststart (moov atom at beginning)
// - H.264 (AVC) video codec, yuv420p pixel format, main profile, level 4.0
// - Even width & height dimensions
// - AAC audio codec, 44.1kHz or 48kHz stereo/mono
func EnsureWhatsAppVideo(ctx context.Context, inputPath, outputPath string) error {
	cmd := exec.CommandContext(ctx, "ffmpeg", "-y", "-hide_banner", "-loglevel", "error",
		"-i", inputPath,
		"-map", "0:v:0",
		"-map", "0:a?",
		"-c:v", "libx264",
		"-preset", "fast",
		"-crf", "23",
		"-pix_fmt", "yuv420p",
		"-profile:v", "main",
		"-level:v", "4.0",
		"-vf", "scale=trunc(iw/2)*2:trunc(ih/2)*2",
		"-c:a", "aac",
		"-b:a", "128k",
		"-movflags", "+faststart",
		outputPath,
	)
	if out, err := cmd.CombinedOutput(); err != nil {
		return fmt.Errorf("ffmpeg video transcode error: %w (%s)", err, strings.TrimSpace(string(out)))
	}
	return nil
}

// EnsureWhatsAppOpus converts an audio file into an OGG Opus stream suitable for WhatsApp.
func EnsureWhatsAppOpus(ctx context.Context, inputPath, outputPath string) error {
	cmd := exec.CommandContext(ctx, "ffmpeg", "-y", "-hide_banner", "-loglevel", "error",
		"-i", inputPath,
		"-vn",
		"-c:a", "libopus",
		"-b:a", "48k",
		"-vbr", "on",
		"-compression_level", "10",
		"-application", "voip",
		"-ar", "48000",
		"-ac", "1",
		"-f", "ogg",
		outputPath,
	)
	if out, err := cmd.CombinedOutput(); err != nil {
		return fmt.Errorf("ffmpeg audio transcode error: %w (%s)", err, strings.TrimSpace(string(out)))
	}
	return nil
}

// sendOpusAudioPayload uploads and dispatches Opus audio with waveform & PTT metadata.
func sendOpusAudioPayload(ctx *dispatch.Context, audioBytes []byte) error {
	meta, err := media.OpusPTTConvert(ctx.GetSendContext(), audioBytes)
	if err != nil || meta == nil || len(meta.Data) == 0 {
		meta = &media.AudioPTTMeta{
			Data: audioBytes,
		}
	}

	uploaded, errUpload := ctx.Client.Upload(ctx.GetSendContext(), meta.Data, whatsmeow.MediaAudio)
	if errUpload != nil {
		// Fallback to standard context reply
		return ctx.ReplyWithAudio(meta.Data, "audio/ogg; codecs=opus")
	}

	ptt := true
	mimetype := "audio/ogg; codecs=opus"
	msg := &waE2E.Message{
		AudioMessage: &waE2E.AudioMessage{
			URL:           &uploaded.URL,
			DirectPath:    &uploaded.DirectPath,
			MediaKey:      uploaded.MediaKey,
			Mimetype:      &mimetype,
			FileEncSHA256: uploaded.FileEncSHA256,
			FileSHA256:    uploaded.FileSHA256,
			FileLength:    proto.Uint64(uint64(len(meta.Data))),
			PTT:           &ptt,
			Seconds:       &meta.Seconds,
			Waveform:      meta.Waveform,
			ContextInfo:   ctx.ReplyContextInfo(),
		},
	}

	_, errSend := ctx.Client.SendMessage(ctx.GetSendContext(), ctx.Chat, msg)
	return errSend
}

// buildCaption formats a clean caption with topic and description context for media messages.
func buildCaption(meta *MediaMeta) string {
	return cleanTopicAndContext(meta.GetTitle(), meta.Description)
}

func formatDurationSeconds(sec float64) string {
	if sec <= 0 {
		return ""
	}
	total := int(sec)
	m := total / 60
	s := total % 60
	if m >= 60 {
		h := m / 60
		m = m % 60
		return fmt.Sprintf("%d:%02d:%02d", h, m, s)
	}
	return fmt.Sprintf("%d:%02d", m, s)
}

func isImageURL(raw string) bool {
	u, err := url.Parse(raw)
	if err != nil {
		return false
	}
	path := strings.ToLower(u.Path)
	return strings.HasSuffix(path, ".jpg") ||
		strings.HasSuffix(path, ".jpeg") ||
		strings.HasSuffix(path, ".png") ||
		strings.HasSuffix(path, ".webp") ||
		strings.HasSuffix(path, ".gif") ||
		strings.HasSuffix(path, ".bmp")
}

func isDirectImage(ext string) bool {
	clean := strings.ToLower(strings.TrimPrefix(ext, "."))
	return clean == "jpg" || clean == "jpeg" || clean == "png" || clean == "webp"
}

func sendUsage(ctx *dispatch.Context) error {
	p := ctx.GetPrefix()
	tb := dispatch.NewText()
	tb.Line("*Omni-Media Downloader (yt-dlp)*").Blank()
	tb.Line("*Usage:*")
	tb.Linef("- %sdl <url>  (Fetches media and sends interactive Video/Audio poll)", p)
	tb.Linef("- %sdl <url> video  (Directly downloads video)", p)
	tb.Linef("- %sdl <url> audio  (Directly downloads audio as WhatsApp Opus)", p)
	tb.Linef("- Reply to any message containing a link with *%sdl*", p)
	tb.Blank()
	tb.Line("*Cookies Management:*")
	tb.Linef("- Reply to a `cookies.txt` document with *%sdl cookie*", p)
	tb.Linef("- %sdl cookie <cookie text>", p)
	tb.Linef("- %sdl cookies  (Check cookies status)", p)
	tb.Linef("- %sdl cookie clear  (Remove stored cookies)", p)
	tb.Blank()
	tb.Line("*Features:*")
	tb.Line("- Automatic image detection (Instagram, Reddit, X/Twitter, direct links)")
	tb.Line("- WhatsApp-compatible H.264 video with faststart streaming")
	tb.Line("- Voice note Opus OGG audio with interactive waveform")

	return ctx.Reply(tb.String())
}
