package whatsrook

import (
	"whatsrook/util/botctx"
	"whatsrook/util/send"
)

// AlbumMediaItem represents a media payload item to be bundled in a WhatsApp album.
type AlbumMediaItem = send.AlbumMediaItem

// Sender encapsulates message sending capabilities.
type Sender = send.Sender

var (
	// NewSender creates a new standalone Sender instance.
	NewSender = send.New
	// ResolveMentionJIDStrings resolves mentions to string list.
	ResolveMentionJIDStrings = send.ResolveMentionJIDStrings
	// SendText sends a plain text message.
	SendText = send.Text
	// SendTextWithID sends a plain text message returning message ID.
	SendTextWithID = send.TextWithID
	// SendTextWithMentions sends a text message with mentions.
	SendTextWithMentions = send.TextWithMentions
	// SendReply sends a quoted reply.
	SendReply = send.Reply
	// SendReplyWithID sends a quoted reply returning message ID.
	SendReplyWithID = send.ReplyWithID
	// SendReplyWithMentions sends a quoted reply with mentions.
	SendReplyWithMentions = send.ReplyWithMentions
	// SendImage sends an image.
	SendImage = send.Image
	// SendImageWithMentions sends an image with mentions.
	SendImageWithMentions = send.ImageWithMentions
	// SendVideo sends a video.
	SendVideo = send.Video
	// SendVideoGif sends a video GIF.
	SendVideoGif = send.VideoGif
	// SendVideoWithMentions sends a video with mentions.
	SendVideoWithMentions = send.VideoWithMentions
	// SendAudio sends audio.
	SendAudio = send.Audio
	// SendDocument sends a document.
	SendDocument = send.Document
	// SendSticker sends a sticker.
	SendSticker = send.Sticker
	// SendAlbum sends an album.
	SendAlbum = send.Album
	// SendReact sends a reaction.
	SendReact = send.React
	// SendReactMessage sends a reaction by message ID.
	SendReactMessage = send.ReactMessage
	// SendEdit edits a message.
	SendEdit = send.Edit
	// SendDelete revokes a message.
	SendDelete = send.Delete

	// DispatchListSelection resolves an interactive list button response.
	DispatchListSelection = botctx.DispatchListSelection
	// DispatchPollVoteEvent routes incoming poll votes to reactive action handlers.
	DispatchPollVoteEvent = botctx.DispatchPollVoteEvent
)
