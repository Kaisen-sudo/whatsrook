package whatsrook

import (
	"context"

	"go.mau.fi/whatsmeow"

	"whatsrook/util/botctx"
	"whatsrook/util/message"
)

// SettingGetter retrieves a setting value for a client and key from the database store.
type SettingGetter = botctx.SettingGetter

// SettingSetter updates a setting value for a client and key in the database store.
type SettingSetter = botctx.SettingSetter

// SettingDeleter removes a setting value for a client and key from the database store.
type SettingDeleter = botctx.SettingDeleter

// RecentMessageCache keeps an in-memory cache of recent incoming message events per JID.
type RecentMessageCache = message.RecentMessageCache

var (
	GlobalSettingGetter  SettingGetter
	GlobalSettingSetter  SettingSetter
	GlobalSettingDeleter SettingDeleter
)

func init() {
	botctx.GlobalSettingGetter = func(ctx context.Context, client *whatsmeow.Client, key string) (string, error) {
		if GlobalSettingGetter != nil {
			return GlobalSettingGetter(ctx, client, key)
		}
		return "", nil
	}
	botctx.GlobalSettingSetter = func(ctx context.Context, client *whatsmeow.Client, key, value string) error {
		if GlobalSettingSetter != nil {
			return GlobalSettingSetter(ctx, client, key, value)
		}
		return nil
	}
	botctx.GlobalSettingDeleter = func(ctx context.Context, client *whatsmeow.Client, key string) error {
		if GlobalSettingDeleter != nil {
			return GlobalSettingDeleter(ctx, client, key)
		}
		return nil
	}
}

// Re-exports from message and botctx
var (
	GetMediaType                 = message.GetMediaType
	ExtractMessageText           = message.ExtractMessageText
	ExtractTextFromProto         = message.ExtractTextFromProto
	ExtractMediaFromEvent        = message.ExtractMediaFromEvent
	ExtractMedia                 = message.ExtractMedia
	UnwrapMessageProto           = message.UnwrapMessageProto
	GetContextInfoFromProto      = message.GetContextInfoFromProto
	AttachContextInfo            = message.AttachContextInfo
	StripContextInfo             = message.StripContextInfo
	IsViewOnceMessage            = message.IsViewOnceMessage
	ExtractViewOnceMessage       = message.ExtractViewOnceMessage
	UnwrapAndSendViewOnceMessage = message.UnwrapAndSendViewOnceMessage
	RemoveEmojis                 = message.RemoveEmojis
	FormatTextResponseRaw        = message.FormatTextResponseRaw
	EncodeProtoMessage           = message.EncodeProtoMessage
	DecodeProtoMessage           = message.DecodeProtoMessage
	RecordRecentMessage          = message.RecordRecentMessage
	GetRecentMessageForJID       = message.GetRecentMessageForJID
	GlobalRecentMessages         = message.GlobalRecentMessages

	IsSudoRaw              = botctx.IsSudoRaw
	IsAdminRaw             = botctx.IsAdminRaw
	IsBotAdminRaw          = botctx.IsBotAdminRaw
	IsSameUserRaw          = botctx.IsSameUserRaw
	ParticipantMatchesUser = botctx.ParticipantMatchesUser
	ResolveMentionJIDs     = botctx.ResolveMentionJIDs
	ResolveMentionRaw      = botctx.ResolveMentionRaw
	ResolveContactName     = botctx.ResolveContactName
	GetClientSetting       = botctx.GetClientSetting
	PutClientSetting       = botctx.PutClientSetting
	DeleteClientSetting    = botctx.DeleteClientSetting
)
