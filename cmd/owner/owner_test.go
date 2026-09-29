package owner

import (
	"os"
	"path/filepath"
	"testing"
)

func TestShellWorkingDirPersistence(t *testing.T) {
	tempDir, err := os.MkdirTemp("", "whatsrook-sh-test-*")
	if err != nil {
		t.Fatalf("failed to create temp dir: %v", err)
	}
	defer os.RemoveAll(tempDir)

	subDir := filepath.Join(tempDir, "sub_bin")
	if err := os.Mkdir(subDir, 0755); err != nil {
		t.Fatalf("failed to create sub dir: %v", err)
	}

	chatKey := "test_chat_1"

	// 1. Initial directory should be valid and fallback to os.Getwd()
	initial := getShellWorkingDir(chatKey)
	if initial == "" {
		t.Fatalf("expected non-empty initial working dir")
	}

	// 2. Change working directory
	setShellWorkingDir(chatKey, subDir)
	current := getShellWorkingDir(chatKey)
	if current != subDir {
		t.Fatalf("expected working dir %q, got %q", subDir, current)
	}

	// 3. Different chat should have its own independent directory
	otherChatKey := "test_chat_2"
	otherDir := getShellWorkingDir(otherChatKey)
	if otherDir == subDir {
		t.Fatalf("expected different chat to not share modified directory without being set")
	}

	// 4. If directory is removed, it should safely fallback to default
	_ = os.Remove(subDir)
	fallback := getShellWorkingDir(chatKey)
	if fallback == subDir {
		t.Fatalf("expected fallback when directory was removed, got %q", fallback)
	}
}

func TestCleanShellOutput(t *testing.T) {
	raw := "Hello\x1b[31m World\x1b[0m\rOverwritten!\nLine 2\r\nLine 3"
	cleaned := CleanShellOutput(raw)
	if cleaned == "" {
		t.Fatalf("expected non-empty cleaned output")
	}
	if cleaned != "Overwritten!\nLine 2\nLine 3" {
		t.Errorf("unexpected output: %q", cleaned)
	}
}
