package whatsrook

import (
	"encoding/hex"
	"fmt"
	"time"

	"go.mau.fi/whatsmeow"
	"go.mau.fi/whatsmeow/proto/waE2E"
	"go.mau.fi/whatsmeow/types"
	"go.mau.fi/whatsmeow/types/events"

	"whatsrook/util"
	"whatsrook/util/builder"
	"whatsrook/util/logger"
)

// SendText sends a plain text message without quoting.
func (c *PluginContext) SendText(text string) error {
	c.StopAutoLoader()
	if c.Client == nil {
		return fmt.Errorf("client unavailable")
	}
	formatted := c.formatTextResponse(text)
	_, err := c.Client.SendMessage(c.GetSendContext(), c.Chat, &waE2E.Message{
		Conversation: &formatted,
	})
	return err
}

func (c *PluginContext) resolveMentionJIDStrings(mentions []types.JID) []string {
	if len(mentions) == 0 {
		return nil
	}
	ctx := c.GetSendContext()
	seen := make(map[string]bool)
	var mentionStrs []string
	for _, m := range mentions {
		if m.IsEmpty() {
			continue
		}
		norm := m.ToNonAD()
		key := norm.String()
		if !seen[key] {
			seen[key] = true
			mentionStrs = append(mentionStrs, key)
		}
		if c.Client != nil && c.Client.Store != nil && c.Client.Store.LIDs != nil {
			switch norm.Server {
			case types.HiddenUserServer:
				if pn, err := c.Client.Store.LIDs.GetPNForLID(ctx, norm); err == nil && !pn.IsEmpty() {
					pnStr := pn.ToNonAD().String()
					if !seen[pnStr] {
						seen[pnStr] = true
						mentionStrs = append(mentionStrs, pnStr)
					}
				}
			case types.DefaultUserServer:
				if lid, err := c.Client.Store.LIDs.GetLIDForPN(ctx, norm); err == nil && !lid.IsEmpty() {
					lidStr := lid.ToNonAD().String()
					if !seen[lidStr] {
						seen[lidStr] = true
						mentionStrs = append(mentionStrs, lidStr)
					}
				}
			}
		}
	}
	return mentionStrs
}

// SendTextWithMentions sends a text message with mentioned JIDs without quoting.
func (c *PluginContext) SendTextWithMentions(text string, mentions []types.JID) error {
	c.StopAutoLoader()
	if c.Client == nil {
		return fmt.Errorf("client unavailable")
	}
	formatted := c.formatTextResponse(text)
	mentionStrs := c.resolveMentionJIDStrings(mentions)
	_, err := c.Client.SendMessage(c.GetSendContext(), c.Chat, &waE2E.Message{
		ExtendedTextMessage: &waE2E.ExtendedTextMessage{
			Text: &formatted,
			ContextInfo: &waE2E.ContextInfo{
				MentionedJID: mentionStrs,
			},
		},
	})
	return err
}

// SendImage sends an image without quoting.
func (c *PluginContext) SendImage(data []byte, mimetype, caption string) error {
	return c.SendImageWithMentions(data, mimetype, caption, nil)
}

// SendImageWithMentions sends an image with mentions without quoting.
func (c *PluginContext) SendImageWithMentions(data []byte, mimetype, caption string, mentions []types.JID) error {
	c.StopAutoLoader()
	if c.Client == nil {
		return fmt.Errorf("client unavailable")
	}
	if mimetype == "" {
		mimetype = "image/jpeg"
	}
	uploaded, err := c.Client.Upload(c.GetSendContext(), data, whatsmeow.MediaImage)
	if err != nil {
		return fmt.Errorf("upload image failed: %w", err)
	}

	var ci *waE2E.ContextInfo
	if mentionStrs := c.resolveMentionJIDStrings(mentions); len(mentionStrs) > 0 {
		ci = &waE2E.ContextInfo{
			MentionedJID: mentionStrs,
		}
	}

	msg := &waE2E.Message{
		ImageMessage: &waE2E.ImageMessage{
			URL:           &uploaded.URL,
			DirectPath:    &uploaded.DirectPath,
			MediaKey:      uploaded.MediaKey,
			Mimetype:      &mimetype,
			FileEncSHA256: uploaded.FileEncSHA256,
			FileSHA256:    uploaded.FileSHA256,
			FileLength:    new(uint64(len(data))),
			Caption:       &caption,
			ContextInfo:   ci,
		},
	}
	_, err = c.Client.SendMessage(c.GetSendContext(), c.Chat, msg)
	return err
}

// SendVideo sends a video without quoting.
func (c *PluginContext) SendVideo(data []byte, mimetype, caption string) error {
	return c.sendVideoInternal(data, mimetype, caption, false)
}

// SendVideoGif sends a looping GIF video without quoting.
func (c *PluginContext) SendVideoGif(data []byte, mimetype, caption string) error {
	return c.sendVideoInternal(data, mimetype, caption, true)
}

// SendVideoWithMentions sends a video without quoting, mentioning specific user JIDs.
func (c *PluginContext) SendVideoWithMentions(data []byte, mimetype, caption string, mentions []types.JID) error {
	c.StopAutoLoader()
	if c.Client == nil {
		return fmt.Errorf("client unavailable")
	}
	if mimetype == "" {
		mimetype = "video/mp4"
	}
	uploaded, err := c.Client.Upload(c.GetSendContext(), data, whatsmeow.MediaVideo)
	if err != nil {
		return fmt.Errorf("upload video failed: %w", err)
	}

	var ci *waE2E.ContextInfo
	if mentionStrs := c.resolveMentionJIDStrings(mentions); len(mentionStrs) > 0 {
		ci = &waE2E.ContextInfo{
			MentionedJID: mentionStrs,
		}
	}

	msg := &waE2E.Message{
		VideoMessage: &waE2E.VideoMessage{
			URL:           &uploaded.URL,
			DirectPath:    &uploaded.DirectPath,
			MediaKey:      uploaded.MediaKey,
			Mimetype:      &mimetype,
			FileEncSHA256: uploaded.FileEncSHA256,
			FileSHA256:    uploaded.FileSHA256,
			FileLength:    new(uint64(len(data))),
			Caption:       &caption,
			ContextInfo:   ci,
		},
	}
	_, err = c.Client.SendMessage(c.GetSendContext(), c.Chat, msg)
	return err
}

func (c *PluginContext) sendVideoInternal(data []byte, mimetype, caption string, isGif bool) error {
	c.StopAutoLoader()
	if c.Client == nil {
		return fmt.Errorf("client unavailable")
	}
	if mimetype == "" {
		mimetype = "video/mp4"
	}
	uploaded, err := c.Client.Upload(c.GetSendContext(), data, whatsmeow.MediaVideo)
	if err != nil {
		return fmt.Errorf("upload video failed: %w", err)
	}
	msg := &waE2E.Message{
		VideoMessage: &waE2E.VideoMessage{
			URL:           &uploaded.URL,
			DirectPath:    &uploaded.DirectPath,
			MediaKey:      uploaded.MediaKey,
			Mimetype:      &mimetype,
			GifPlayback:   new(isGif),
			FileEncSHA256: uploaded.FileEncSHA256,
			FileSHA256:    uploaded.FileSHA256,
			FileLength:    new(uint64(len(data))),
			Caption:       &caption,
		},
	}
	_, err = c.Client.SendMessage(c.GetSendContext(), c.Chat, msg)
	return err
}

// SendAudio sends an audio file without quoting.
func (c *PluginContext) SendAudio(data []byte, mimetype string) error {
	c.StopAutoLoader()
	if c.Client == nil {
		return fmt.Errorf("client unavailable")
	}
	if mimetype == "" {
		mimetype = "audio/mp4"
	}
	uploaded, err := c.Client.Upload(c.GetSendContext(), data, whatsmeow.MediaAudio)
	if err != nil {
		return fmt.Errorf("upload audio failed: %w", err)
	}
	msg := &waE2E.Message{
		AudioMessage: &waE2E.AudioMessage{
			URL:           &uploaded.URL,
			DirectPath:    &uploaded.DirectPath,
			MediaKey:      uploaded.MediaKey,
			Mimetype:      &mimetype,
			FileEncSHA256: uploaded.FileEncSHA256,
			FileSHA256:    uploaded.FileSHA256,
			FileLength:    new(uint64(len(data))),
		},
	}
	_, err = c.Client.SendMessage(c.GetSendContext(), c.Chat, msg)
	return err
}

// SendDocument sends a document without quoting.
func (c *PluginContext) SendDocument(data []byte, mimetype, filename, caption string) error {
	c.StopAutoLoader()
	if c.Client == nil {
		return fmt.Errorf("client unavailable")
	}
	if mimetype == "" {
		mimetype = "application/octet-stream"
	}
	uploaded, err := c.Client.Upload(c.GetSendContext(), data, whatsmeow.MediaDocument)
	if err != nil {
		return fmt.Errorf("upload document failed: %w", err)
	}
	msg := &waE2E.Message{
		DocumentMessage: &waE2E.DocumentMessage{
			URL:           &uploaded.URL,
			DirectPath:    &uploaded.DirectPath,
			MediaKey:      uploaded.MediaKey,
			Mimetype:      &mimetype,
			FileName:      &filename,
			FileEncSHA256: uploaded.FileEncSHA256,
			FileSHA256:    uploaded.FileSHA256,
			FileLength:    new(uint64(len(data))),
			Caption:       &caption,
		},
	}
	_, err = c.Client.SendMessage(c.GetSendContext(), c.Chat, msg)
	return err
}

// SendSticker sends a WebP sticker without quoting.
func (c *PluginContext) SendSticker(data []byte) error {
	c.StopAutoLoader()
	if c.Client == nil {
		return fmt.Errorf("client unavailable")
	}

	if len(data) < 12 || string(data[0:4]) != "RIFF" || string(data[8:12]) != "WEBP" {
		logger.Error("SendSticker: invalid payload data", "bytes", len(data))
		return fmt.Errorf("invalid sticker data: missing WebP header")
	}

	if meta, err := util.GetStickerMetadata(data); err != nil || meta == nil {
		pack := c.GetStickerPack()
		author := c.GetStickerAuthor()
		logger.Debug("SendSticker: injecting clean WebP sticker metadata", "pack", pack, "author", author)
		if withMeta, err := util.AddStickerMetadata(data, pack, author); err == nil {
			data = withMeta
		} else {
			logger.Warn("SendSticker: could not inject sticker metadata", "err", err)
		}
	}

	uploaded, err := c.Client.Upload(c.GetSendContext(), data, whatsmeow.MediaImage)
	if err != nil {
		return fmt.Errorf("upload sticker failed: %w", err)
	}

	width := uint32(512)
	height := uint32(512)
	isAnimated := false
	mimetype := "image/webp"

	logger.Debug("SendSticker: outgoing StickerMessage debug info",
		"chat", c.Chat.String(),
		"bytes", len(data),
		"url", uploaded.URL,
		"direct_path", uploaded.DirectPath,
		"file_sha256", hex.EncodeToString(uploaded.FileSHA256),
		"file_enc_sha256", hex.EncodeToString(uploaded.FileEncSHA256),
		"media_key_len", len(uploaded.MediaKey),
		"width", width,
		"height", height,
		"mimetype", mimetype,
	)

	msg := &waE2E.Message{
		StickerMessage: &waE2E.StickerMessage{
			URL:           &uploaded.URL,
			DirectPath:    &uploaded.DirectPath,
			MediaKey:      uploaded.MediaKey,
			Mimetype:      &mimetype,
			FileEncSHA256: uploaded.FileEncSHA256,
			FileSHA256:    uploaded.FileSHA256,
			FileLength:    new(uint64(len(data))),
			Width:         &width,
			Height:        &height,
			IsAnimated:    &isAnimated,
		},
	}
	_, err = c.Client.SendMessage(c.GetSendContext(), c.Chat, msg)
	return err
}

// SendTextWithGroupMention sends a text message with WhatsApp native group mention.
func (c *PluginContext) SendTextWithGroupMention(text string) error {
	c.StopAutoLoader()
	if c.Client == nil {
		return fmt.Errorf("client unavailable")
	}
	formatted := c.formatTextResponse(text)
	var nonJID uint32 = 1
	msg := &waE2E.Message{
		ExtendedTextMessage: &waE2E.ExtendedTextMessage{
			Text: &formatted,
			ContextInfo: &waE2E.ContextInfo{
				NonJIDMentions: &nonJID,
			},
		},
	}
	_, err := c.Client.SendMessage(c.GetSendContext(), c.Chat, msg)
	return err
}

// Reply sends a plain text response quoted to the triggering message.
func (c *PluginContext) Reply(text string) error {
	_, err := c.ReplyWithID(text)
	return err
}

// Replyf formats and sends a text response quoted to the triggering message.
func (c *PluginContext) Replyf(format string, args ...any) error {
	return c.Reply(fmt.Sprintf(format, args...))
}

// ReplyWithMentions sends a text message with mentions quoted to the triggering message.
func (c *PluginContext) ReplyWithMentions(text string, mentions []types.JID) error {
	c.StopAutoLoader()
	if c.Client == nil {
		return fmt.Errorf("client unavailable")
	}
	formatted := c.formatTextResponse(text)
	ci := c.replyContextInfo()
	if ci == nil {
		ci = &waE2E.ContextInfo{}
	}
	ci.MentionedJID = append(ci.MentionedJID, c.resolveMentionJIDStrings(mentions)...)
	msg := &waE2E.Message{
		ExtendedTextMessage: &waE2E.ExtendedTextMessage{
			Text:        &formatted,
			ContextInfo: ci,
		},
	}
	_, err := c.Client.SendMessage(c.GetSendContext(), c.Chat, msg)
	return err
}

// ReplyWithID sends a text message and returns the sent message ID.
func (c *PluginContext) ReplyWithID(text string) (string, error) {
	c.StopAutoLoader()
	if c.Client == nil {
		return "", fmt.Errorf("client unavailable")
	}
	ctx := c.GetSendContext()
	formatted := c.formatTextResponse(text)
	msg := &waE2E.Message{
		ExtendedTextMessage: &waE2E.ExtendedTextMessage{
			Text:        &formatted,
			ContextInfo: c.replyContextInfo(),
		},
	}
	resp, err := c.Client.SendMessage(ctx, c.Chat, msg)
	if err != nil {
		return "", err
	}
	return resp.ID, nil
}

// SendTextWithID sends a text message without quoting and returns the sent message ID.
func (c *PluginContext) SendTextWithID(text string) (string, error) {
	c.StopAutoLoader()
	if c.Client == nil {
		return "", fmt.Errorf("client unavailable")
	}
	ctx := c.GetSendContext()
	formatted := c.formatTextResponse(text)
	msg := &waE2E.Message{
		Conversation: &formatted,
	}
	resp, err := c.Client.SendMessage(ctx, c.Chat, msg)
	if err != nil {
		return "", err
	}
	return resp.ID, nil
}

// ReplyWithImage uploads and sends an image quoted to the triggering message.
func (c *PluginContext) ReplyWithImage(data []byte, mimetype, caption string) error {
	return c.ReplyWithImageWithMentions(data, mimetype, caption, nil)
}

// ReplyWithImageWithMentions uploads and sends an image with mentions quoted to the triggering message.
func (c *PluginContext) ReplyWithImageWithMentions(data []byte, mimetype, caption string, mentions []types.JID) error {
	c.StopAutoLoader()
	if c.Client == nil {
		return fmt.Errorf("client unavailable")
	}
	if mimetype == "" {
		mimetype = "image/jpeg"
	}
	uploaded, err := c.Client.Upload(c.GetSendContext(), data, whatsmeow.MediaImage)
	if err != nil {
		return fmt.Errorf("upload image failed: %w", err)
	}

	ci := c.replyContextInfo()
	if ci == nil && len(mentions) > 0 {
		ci = &waE2E.ContextInfo{}
	}
	if ci != nil {
		ci.MentionedJID = append(ci.MentionedJID, c.resolveMentionJIDStrings(mentions)...)
	}

	msg := &waE2E.Message{
		ImageMessage: &waE2E.ImageMessage{
			URL:           &uploaded.URL,
			DirectPath:    &uploaded.DirectPath,
			MediaKey:      uploaded.MediaKey,
			Mimetype:      &mimetype,
			FileEncSHA256: uploaded.FileEncSHA256,
			FileSHA256:    uploaded.FileSHA256,
			FileLength:    new(uint64(len(data))),
			Caption:       &caption,
			ContextInfo:   ci,
		},
	}
	_, err = c.Client.SendMessage(c.GetSendContext(), c.Chat, msg)
	return err
}

// ReplyWithVideo uploads and sends a video quoted to the triggering message.
func (c *PluginContext) ReplyWithVideo(data []byte, mimetype, caption string) error {
	return c.replyVideoInternal(data, mimetype, caption, false)
}

// ReplyWithVideoGif uploads and sends a GIF video quoted to the triggering message.
func (c *PluginContext) ReplyWithVideoGif(data []byte, mimetype, caption string) error {
	return c.replyVideoInternal(data, mimetype, caption, true)
}

func (c *PluginContext) replyVideoInternal(data []byte, mimetype, caption string, isGif bool) error {
	c.StopAutoLoader()
	if c.Client == nil {
		return fmt.Errorf("client unavailable")
	}
	if mimetype == "" {
		mimetype = "video/mp4"
	}
	uploaded, err := c.Client.Upload(c.GetSendContext(), data, whatsmeow.MediaVideo)
	if err != nil {
		return fmt.Errorf("upload video failed: %w", err)
	}

	msg := &waE2E.Message{
		VideoMessage: &waE2E.VideoMessage{
			URL:           &uploaded.URL,
			DirectPath:    &uploaded.DirectPath,
			MediaKey:      uploaded.MediaKey,
			Mimetype:      &mimetype,
			GifPlayback:   new(isGif),
			FileEncSHA256: uploaded.FileEncSHA256,
			FileSHA256:    uploaded.FileSHA256,
			FileLength:    new(uint64(len(data))),
			Caption:       &caption,
			ContextInfo:   c.replyContextInfo(),
		},
	}
	_, err = c.Client.SendMessage(c.GetSendContext(), c.Chat, msg)
	return err
}

// ReplyWithVideoWithMentions uploads and sends video quoted with mentioned user JIDs.
func (c *PluginContext) ReplyWithVideoWithMentions(data []byte, mimetype, caption string, mentions []types.JID) error {
	c.StopAutoLoader()
	if c.Client == nil {
		return fmt.Errorf("client unavailable")
	}
	if mimetype == "" {
		mimetype = "video/mp4"
	}
	uploaded, err := c.Client.Upload(c.GetSendContext(), data, whatsmeow.MediaVideo)
	if err != nil {
		return fmt.Errorf("upload video failed: %w", err)
	}

	ci := c.replyContextInfo()
	if ci == nil {
		ci = &waE2E.ContextInfo{}
	}
	ci.MentionedJID = c.resolveMentionJIDStrings(mentions)

	msg := &waE2E.Message{
		VideoMessage: &waE2E.VideoMessage{
			URL:           &uploaded.URL,
			DirectPath:    &uploaded.DirectPath,
			MediaKey:      uploaded.MediaKey,
			Mimetype:      &mimetype,
			FileEncSHA256: uploaded.FileEncSHA256,
			FileSHA256:    uploaded.FileSHA256,
			FileLength:    new(uint64(len(data))),
			Caption:       &caption,
			ContextInfo:   ci,
		},
	}
	_, err = c.Client.SendMessage(c.GetSendContext(), c.Chat, msg)
	return err
}

// AlbumMediaItem represents a media item (image or video) to be bundled in an album message.
type AlbumMediaItem struct {
	Data     []byte
	Mimetype string
	Caption  string
	IsVideo  bool
}

// SendAlbum sends multiple media items grouped as a WhatsApp album without quoting.
func (c *PluginContext) SendAlbum(items []AlbumMediaItem) error {
	return c.sendAlbumInternal(items, false)
}

// ReplyWithAlbum sends multiple media items grouped as a WhatsApp album quoted to the triggering message.
func (c *PluginContext) ReplyWithAlbum(items []AlbumMediaItem) error {
	return c.sendAlbumInternal(items, true)
}

func (c *PluginContext) sendAlbumInternal(items []AlbumMediaItem, isReply bool) error {
	c.StopAutoLoader()
	if c.Client == nil {
		return fmt.Errorf("client unavailable")
	}

	var validItems []AlbumMediaItem
	for _, it := range items {
		if len(it.Data) > 0 {
			validItems = append(validItems, it)
		}
	}

	if len(validItems) == 0 {
		return fmt.Errorf("no media items to send in album")
	}

	// Single item delivery does not require an album wrapper.
	if len(validItems) == 1 {
		it := validItems[0]
		if it.IsVideo {
			if isReply {
				return c.ReplyWithVideo(it.Data, it.Mimetype, it.Caption)
			}
			return c.SendVideo(it.Data, it.Mimetype, it.Caption)
		}
		if isReply {
			return c.ReplyWithImage(it.Data, it.Mimetype, it.Caption)
		}
		return c.SendImage(it.Data, it.Mimetype, it.Caption)
	}

	ctx := c.GetSendContext()

	type uploadedAlbumItem struct {
		item     AlbumMediaItem
		uploaded whatsmeow.UploadResponse
	}

	var uploadedItems []uploadedAlbumItem
	for _, it := range validItems {
		mediaType := whatsmeow.MediaImage
		if it.IsVideo {
			mediaType = whatsmeow.MediaVideo
		}
		up, err := c.Client.Upload(ctx, it.Data, mediaType)
		if err != nil {
			logger.Error("sendAlbum: failed to upload media item", "isVideo", it.IsVideo, "bytes", len(it.Data), "err", err)
			continue
		}
		uploadedItems = append(uploadedItems, uploadedAlbumItem{
			item:     it,
			uploaded: up,
		})
	}

	if len(uploadedItems) == 0 {
		return fmt.Errorf("failed to upload any album media items")
	}

	// If only 1 item succeeded in uploading, send it directly.
	if len(uploadedItems) == 1 {
		up := uploadedItems[0]
		if up.item.IsVideo {
			if isReply {
				return c.ReplyWithVideo(up.item.Data, up.item.Mimetype, up.item.Caption)
			}
			return c.SendVideo(up.item.Data, up.item.Mimetype, up.item.Caption)
		}
		if isReply {
			return c.ReplyWithImage(up.item.Data, up.item.Mimetype, up.item.Caption)
		}
		return c.SendImage(up.item.Data, up.item.Mimetype, up.item.Caption)
	}

	var imgCount, vidCount uint32
	for _, up := range uploadedItems {
		if up.item.IsVideo {
			vidCount++
		} else {
			imgCount++
		}
	}

	var ci *waE2E.ContextInfo
	if isReply {
		ci = c.replyContextInfo()
	}

	albumMsg := &waE2E.Message{
		AlbumMessage: &waE2E.AlbumMessage{
			ExpectedImageCount: &imgCount,
			ExpectedVideoCount: &vidCount,
			ContextInfo:        ci,
		},
	}

	resp, err := c.Client.SendMessage(ctx, c.Chat, albumMsg)
	if err != nil {
		logger.Warn("sendAlbum: failed to send AlbumMessage header, falling back to sequential delivery", "err", err)
		for i, up := range uploadedItems {
			var sendErr error
			if up.item.IsVideo {
				if i == 0 && isReply {
					sendErr = c.ReplyWithVideo(up.item.Data, up.item.Mimetype, up.item.Caption)
				} else {
					sendErr = c.SendVideo(up.item.Data, up.item.Mimetype, up.item.Caption)
				}
			} else {
				if i == 0 && isReply {
					sendErr = c.ReplyWithImage(up.item.Data, up.item.Mimetype, up.item.Caption)
				} else {
					sendErr = c.SendImage(up.item.Data, up.item.Mimetype, up.item.Caption)
				}
			}
			if sendErr != nil {
				logger.Error("sendAlbum fallback: failed to send media item", "index", i, "err", sendErr)
			}
			if i < len(uploadedItems)-1 {
				time.Sleep(300 * time.Millisecond)
			}
		}
		return nil
	}

	parentKey := c.Client.BuildMessageKey(c.Chat, types.EmptyJID, resp.ID)
	assocType := waE2E.MessageAssociation_MEDIA_ALBUM

	for i, up := range uploadedItems {
		mimetype := up.item.Mimetype
		caption := up.item.Caption
		var captionPtr *string
		if caption != "" {
			captionPtr = &caption
		}

		childMsg := &waE2E.Message{
			MessageContextInfo: &waE2E.MessageContextInfo{
				MessageAssociation: &waE2E.MessageAssociation{
					AssociationType:  &assocType,
					ParentMessageKey: parentKey,
					MessageIndex:     new(int32(i)),
				},
			},
		}

		if up.item.IsVideo {
			if mimetype == "" {
				mimetype = "video/mp4"
			}
			childMsg.VideoMessage = &waE2E.VideoMessage{
				URL:           &up.uploaded.URL,
				DirectPath:    &up.uploaded.DirectPath,
				MediaKey:      up.uploaded.MediaKey,
				Mimetype:      &mimetype,
				FileEncSHA256: up.uploaded.FileEncSHA256,
				FileSHA256:    up.uploaded.FileSHA256,
				FileLength:    new(uint64(len(up.item.Data))),
				Caption:       captionPtr,
			}
		} else {
			if mimetype == "" {
				mimetype = "image/jpeg"
			}
			childMsg.ImageMessage = &waE2E.ImageMessage{
				URL:           &up.uploaded.URL,
				DirectPath:    &up.uploaded.DirectPath,
				MediaKey:      up.uploaded.MediaKey,
				Mimetype:      &mimetype,
				FileEncSHA256: up.uploaded.FileEncSHA256,
				FileSHA256:    up.uploaded.FileSHA256,
				FileLength:    new(uint64(len(up.item.Data))),
				Caption:       captionPtr,
			}
		}

		_, sendErr := c.Client.SendMessage(ctx, c.Chat, childMsg)
		if sendErr != nil {
			logger.Error("sendAlbum: failed to send child media item", "index", i, "err", sendErr)
		}

		if i < len(uploadedItems)-1 {
			time.Sleep(150 * time.Millisecond)
		}
	}

	return nil
}

// ReplyWithAudio uploads and sends audio quoted to the triggering message.
func (c *PluginContext) ReplyWithAudio(data []byte, mimetype string) error {
	c.StopAutoLoader()
	if c.Client == nil {
		return fmt.Errorf("client unavailable")
	}
	if mimetype == "" {
		mimetype = "audio/mp4"
	}
	uploaded, err := c.Client.Upload(c.GetSendContext(), data, whatsmeow.MediaAudio)
	if err != nil {
		return fmt.Errorf("upload audio failed: %w", err)
	}

	msg := &waE2E.Message{
		AudioMessage: &waE2E.AudioMessage{
			URL:           &uploaded.URL,
			DirectPath:    &uploaded.DirectPath,
			MediaKey:      uploaded.MediaKey,
			Mimetype:      &mimetype,
			FileEncSHA256: uploaded.FileEncSHA256,
			FileSHA256:    uploaded.FileSHA256,
			FileLength:    new(uint64(len(data))),
			ContextInfo:   c.replyContextInfo(),
		},
	}
	_, err = c.Client.SendMessage(c.GetSendContext(), c.Chat, msg)
	return err
}

// ReplyWithDocument uploads and sends a file document quoted to the triggering message.
func (c *PluginContext) ReplyWithDocument(data []byte, mimetype, filename, caption string) error {
	c.StopAutoLoader()
	if c.Client == nil {
		return fmt.Errorf("client unavailable")
	}
	if mimetype == "" {
		mimetype = "application/octet-stream"
	}
	uploaded, err := c.Client.Upload(c.GetSendContext(), data, whatsmeow.MediaDocument)
	if err != nil {
		return fmt.Errorf("upload document failed: %w", err)
	}

	msg := &waE2E.Message{
		DocumentMessage: &waE2E.DocumentMessage{
			URL:           &uploaded.URL,
			DirectPath:    &uploaded.DirectPath,
			MediaKey:      uploaded.MediaKey,
			Mimetype:      &mimetype,
			FileName:      &filename,
			FileEncSHA256: uploaded.FileEncSHA256,
			FileSHA256:    uploaded.FileSHA256,
			FileLength:    new(uint64(len(data))),
			Caption:       &caption,
			ContextInfo:   c.replyContextInfo(),
		},
	}
	_, err = c.Client.SendMessage(c.GetSendContext(), c.Chat, msg)
	return err
}

// ReplyWithSticker uploads and sends a WebP sticker quoted to the triggering message.
func (c *PluginContext) ReplyWithSticker(data []byte) error {
	c.StopAutoLoader()
	if c.Client == nil {
		return fmt.Errorf("client unavailable")
	}

	if len(data) < 12 || string(data[0:4]) != "RIFF" || string(data[8:12]) != "WEBP" {
		logger.Error("ReplyWithSticker: invalid payload data", "bytes", len(data))
		return fmt.Errorf("invalid sticker data: missing WebP header")
	}

	if meta, err := util.GetStickerMetadata(data); err != nil || meta == nil {
		pack := c.GetStickerPack()
		author := c.GetStickerAuthor()
		logger.Debug("ReplyWithSticker: injecting clean WebP sticker metadata", "pack", pack, "author", author)
		if withMeta, err := util.AddStickerMetadata(data, pack, author); err == nil {
			data = withMeta
		} else {
			logger.Warn("ReplyWithSticker: could not inject sticker metadata", "err", err)
		}
	} else {
		logger.Debug("ReplyWithSticker: existing sticker metadata found", "pack", meta.PackName, "publisher", meta.Publisher)
	}

	logger.Debug("ReplyWithSticker: uploading sticker payload", "chat", c.Chat.String(), "bytes", len(data))
	uploaded, err := c.Client.Upload(c.GetSendContext(), data, whatsmeow.MediaImage)
	if err != nil {
		logger.Error("ReplyWithSticker: upload failed", "chat", c.Chat.String(), "err", err)
		return fmt.Errorf("upload sticker failed: %w", err)
	}
	logger.Debug("ReplyWithSticker: upload succeeded", "chat", c.Chat.String(), "url", uploaded.URL)

	width := uint32(512)
	height := uint32(512)
	isAnimated := false
	mimetype := "image/webp"
	ci := c.replyContextInfo()

	var stanzaID string
	if ci != nil && ci.StanzaID != nil {
		stanzaID = *ci.StanzaID
	}

	logger.Debug("ReplyWithSticker: outgoing StickerMessage debug info",
		"chat", c.Chat.String(),
		"bytes", len(data),
		"url", uploaded.URL,
		"direct_path", uploaded.DirectPath,
		"file_sha256", hex.EncodeToString(uploaded.FileSHA256),
		"file_enc_sha256", hex.EncodeToString(uploaded.FileEncSHA256),
		"media_key_len", len(uploaded.MediaKey),
		"width", width,
		"height", height,
		"mimetype", mimetype,
		"quoted_id", stanzaID,
	)

	msg := &waE2E.Message{
		StickerMessage: &waE2E.StickerMessage{
			URL:           &uploaded.URL,
			DirectPath:    &uploaded.DirectPath,
			MediaKey:      uploaded.MediaKey,
			Mimetype:      &mimetype,
			FileEncSHA256: uploaded.FileEncSHA256,
			FileSHA256:    uploaded.FileSHA256,
			FileLength:    new(uint64(len(data))),
			Width:         &width,
			Height:        &height,
			IsAnimated:    &isAnimated,
			ContextInfo:   ci,
		},
	}
	resp, err := c.Client.SendMessage(c.GetSendContext(), c.Chat, msg)
	if err != nil {
		logger.Error("ReplyWithSticker: SendMessage failed", "chat", c.Chat.String(), "err", err)
	} else {
		logger.Debug("ReplyWithSticker: SendMessage succeeded", "chat", c.Chat.String(), "id", resp.ID)
	}
	return err
}

// ReplyWithGroupMention sends a text message with group mention quoted to the triggering message.
func (c *PluginContext) ReplyWithGroupMention(text string) error {
	c.StopAutoLoader()
	if c.Client == nil {
		return fmt.Errorf("client unavailable")
	}
	formatted := c.formatTextResponse(text)
	var nonJID uint32 = 1
	ci := c.replyContextInfo()
	if ci == nil {
		ci = &waE2E.ContextInfo{}
	}
	ci.NonJIDMentions = &nonJID

	msg := &waE2E.Message{
		ExtendedTextMessage: &waE2E.ExtendedTextMessage{
			Text:        &formatted,
			ContextInfo: ci,
		},
	}
	_, err := c.Client.SendMessage(c.GetSendContext(), c.Chat, msg)
	return err
}

// Edit edits an existing message.
func (c *PluginContext) Edit(msgID types.MessageID, content any, extra ...whatsmeow.SendRequestExtra) (whatsmeow.SendResponse, error) {
	c.StopAutoLoader()
	if c.Client == nil {
		return whatsmeow.SendResponse{}, fmt.Errorf("client unavailable")
	}
	var msg *waE2E.Message
	switch v := content.(type) {
	case string:
		formatted := c.formatTextResponse(v)
		msg = &waE2E.Message{
			Conversation: &formatted,
		}
	case *waE2E.Message:
		msg = v
	default:
		return whatsmeow.SendResponse{}, fmt.Errorf("unsupported content type: %T", content)
	}
	editMsg := c.Client.BuildEdit(c.Chat, msgID, msg)
	return c.Client.SendMessage(c.GetSendContext(), c.Chat, editMsg)
}

// Delete deletes/revokes a message for everyone.
func (c *PluginContext) Delete(msgID types.MessageID, senderJID ...types.JID) (whatsmeow.SendResponse, error) {
	c.StopAutoLoader()
	if c.Client == nil {
		return whatsmeow.SendResponse{}, fmt.Errorf("client unavailable")
	}
	sJID := types.EmptyJID
	if len(senderJID) > 0 {
		sJID = senderJID[0]
	}
	revokeMsg := c.Client.BuildRevoke(c.Chat, sJID, msgID)
	return c.Client.SendMessage(c.GetSendContext(), c.Chat, revokeMsg)
}

// React sends an emoji reaction to the triggering message.
func (c *PluginContext) React(emoji string) error {
	if c.Client == nil {
		return fmt.Errorf("client unavailable")
	}
	targetID := ""
	if c.Evt != nil {
		targetID = c.Evt.Info.ID
	}
	if targetID == "" {
		return fmt.Errorf("no target message for reaction")
	}
	reactionMsg := c.Client.BuildReaction(c.Chat, types.EmptyJID, targetID, emoji)
	_, err := c.Client.SendMessage(c.GetSendContext(), c.Chat, reactionMsg)
	return err
}

// ReactMessage sends an emoji reaction to a specific target message ID.
func (c *PluginContext) ReactMessage(targetID string, emoji string) error {
	if c.Client == nil {
		return fmt.Errorf("client unavailable")
	}
	reactionMsg := c.Client.BuildReaction(c.Chat, types.EmptyJID, targetID, emoji)
	_, err := c.Client.SendMessage(c.GetSendContext(), c.Chat, reactionMsg)
	return err
}

// Text initializes a new TextBuilder bound to this PluginContext.
func (c *PluginContext) Text(initial ...string) *builder.TextBuilder {
	return builder.NewTextWithSender(c, initial...)
}

// NewText initializes a new TextBuilder bound to this PluginContext.
func (c *PluginContext) NewText(initial ...string) *builder.TextBuilder {
	return builder.NewTextWithSender(c, initial...)
}

// Rook returns a WARook builder engine bound to this PluginContext.
func (c *PluginContext) Rook() *builder.WARook {
	return builder.From(c)
}

// Poll initializes a new PollBuilder bound to this PluginContext.
func (c *PluginContext) Poll(question string) *builder.PollBuilder {
	return c.Rook().NewPoll(question)
}

// NewPoll initializes a new PollBuilder bound to this PluginContext.
func (c *PluginContext) NewPoll(question string) *builder.PollBuilder {
	return c.Rook().NewPoll(question)
}

// StartAutoLoader is a no-op retained for interface compatibility.
func (c *PluginContext) StartAutoLoader(delay ...time.Duration) {}

// StopAutoLoader is a no-op retained for interface compatibility.
func (c *PluginContext) StopAutoLoader() {}

var _ builder.Sender = (*PluginContext)(nil)

// DispatchListSelection resolves an interactive list button response.
func DispatchListSelection(ctx any, text, displayText string) bool {
	if sender, ok := ctx.(builder.Sender); ok {
		return builder.DispatchListSelection(sender, text, displayText)
	}
	return false
}

// DispatchPollVoteEvent routes incoming poll votes to reactive action handlers.
func DispatchPollVoteEvent(ctx any, evt *events.Message) bool {
	if sender, ok := ctx.(builder.Sender); ok {
		return builder.DispatchPollVoteEvent(sender, evt)
	}
	return false
}
