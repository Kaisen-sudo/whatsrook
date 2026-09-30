package dl

import (
	"strings"
	"testing"
)

func TestCleanYtdlpOutput(t *testing.T) {
	t.Run("empty output", func(t *testing.T) {
		got := cleanYtdlpOutput("")
		if got != "(starting download...)" {
			t.Fatalf("expected '(starting download...)', got %q", got)
		}
	})

	t.Run("strips ansi and handles carriage returns", func(t *testing.T) {
		raw := "\x1b[0;32m[download]\x1b[0m   0.0% of 10.00MiB at Unknown B/s ETA Unknown\r" +
			"[download]  25.0% of 10.00MiB at 1.50MiB/s ETA 00:05\r" +
			"[download]  50.0% of 10.00MiB at 2.50MiB/s ETA 00:02\n" +
			"[download] 100% of 10.00MiB in 00:04 at 3.33MiB/s\n"

		cleaned := cleanYtdlpOutput(raw)
		if strings.Contains(cleaned, "\x1b") {
			t.Errorf("cleaned output still contains ANSI escapes: %q", cleaned)
		}
		if strings.Contains(cleaned, " 0.0%") || strings.Contains(cleaned, "25.0%") {
			t.Errorf("carriage return parts were not overwritten: %q", cleaned)
		}
		if !strings.Contains(cleaned, "50.0%") || !strings.Contains(cleaned, "100%") {
			t.Errorf("expected final progress lines, got: %q", cleaned)
		}
	})

	t.Run("limits to last five lines", func(t *testing.T) {
		raw := "line 1\nline 2\nline 3\nline 4\nline 5\nline 6\nline 7\n"
		cleaned := cleanYtdlpOutput(raw)
		lines := strings.Split(cleaned, "\n")
		if len(lines) != 5 {
			t.Fatalf("expected 5 lines, got %d: %q", len(lines), lines)
		}
		if lines[0] != "line 3" || lines[4] != "line 7" {
			t.Errorf("unexpected lines kept: %v", lines)
		}
	})
}

func TestSafeMediaTitle(t *testing.T) {
	t.Run("nil meta", func(t *testing.T) {
		if got := safeMediaTitle(nil); got != "Media Download" {
			t.Errorf("expected 'Media Download', got %q", got)
		}
	})

	t.Run("escapes backticks and clamps length", func(t *testing.T) {
		meta := &MediaMeta{
			Title: "Test `code` in title " + strings.Repeat("A", 100),
		}
		got := safeMediaTitle(meta)
		if strings.Contains(got, "`") {
			t.Errorf("backtick not replaced in title: %q", got)
		}
		if len(got) > 75 {
			t.Errorf("title length %d exceeds max 75: %q", len(got), got)
		}
		if !strings.HasSuffix(got, "...") {
			t.Errorf("expected ellipsis suffix, got %q", got)
		}
	})
}

func TestFormatHelpers(t *testing.T) {
	t.Run("formatBytes", func(t *testing.T) {
		if got := formatBytes(500); got != "500B" {
			t.Errorf("expected 500B, got %q", got)
		}
		if got := formatBytes(2048); got != "2.00KiB" {
			t.Errorf("expected 2.00KiB, got %q", got)
		}
		if got := formatBytes(5 * 1024 * 1024); got != "5.00MiB" {
			t.Errorf("expected 5.00MiB, got %q", got)
		}
	})

	t.Run("formatSpeed", func(t *testing.T) {
		if got := formatSpeed(500); got != "500B/s" {
			t.Errorf("expected 500B/s, got %q", got)
		}
		if got := formatSpeed(1500 * 1024); got != "1.46MiB/s" {
			t.Errorf("expected 1.46MiB/s, got %q", got)
		}
	})

	t.Run("formatETA", func(t *testing.T) {
		if got := formatETA(-1); got != "" {
			t.Errorf("expected empty for negative ETA, got %q", got)
		}
		if got := formatETA(45); got != "00:45" {
			t.Errorf("expected 00:45, got %q", got)
		}
		if got := formatETA(125); got != "02:05" {
			t.Errorf("expected 02:05, got %q", got)
		}
		if got := formatETA(3665); got != "01:01:05" {
			t.Errorf("expected 01:01:05, got %q", got)
		}
	})

	t.Run("formatDuration", func(t *testing.T) {
		if got := formatDuration(-5); got != "00:00" {
			t.Errorf("expected 00:00, got %q", got)
		}
		if got := formatDuration(45); got != "00:45" {
			t.Errorf("expected 00:45, got %q", got)
		}
		if got := formatDuration(90); got != "01:30" {
			t.Errorf("expected 01:30, got %q", got)
		}
		if got := formatDuration(3665); got != "01:01:05" {
			t.Errorf("expected 01:01:05, got %q", got)
		}
	})

	t.Run("formatTranscodeProgress", func(t *testing.T) {
		got := formatTranscodeProgress(45.0, 90.0, 2.0)
		if !strings.Contains(got, "[transcode]") || !strings.Contains(got, "50.0%") || !strings.Contains(got, "01:30") || !strings.Contains(got, "2.00x") || !strings.Contains(got, "ETA 00:22") {
			t.Errorf("unexpected transcode progress formatting: %q", got)
		}

		gotUnknownTotal := formatTranscodeProgress(45.0, 0, 2.0)
		if !strings.Contains(gotUnknownTotal, "[transcode]") || !strings.Contains(gotUnknownTotal, "00:45 processed at 2.00x") {
			t.Errorf("unexpected transcode progress for unknown total: %q", gotUnknownTotal)
		}
	})
}
