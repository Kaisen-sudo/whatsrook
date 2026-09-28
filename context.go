package whatsrook

import (
	"whatsrook/util/botctx"
)

// PluginContext captures the invocation execution environment for external/native plugin actions.
type PluginContext = botctx.PluginContext

// Loader represents a no-op loader retained for interface compatibility.
type Loader = botctx.Loader

var (
	// Bold returns plain text without WhatsApp bold formatting symbols (*).
	Bold = botctx.Bold
	// Boldf formats text according to format specifier without bold formatting symbols.
	Boldf = botctx.Boldf
	// Italic returns plain text without WhatsApp italic formatting symbols (_).
	Italic = botctx.Italic
	// Italicf formats text according to format specifier without italic formatting symbols.
	Italicf = botctx.Italicf
	// Code returns plain text without WhatsApp inline code formatting symbols (`).
	Code = botctx.Code
	// Codef formats text according to format specifier without inline code formatting symbols.
	Codef = botctx.Codef
	// CodeBlock returns plain text without WhatsApp code block formatting symbols (```).
	CodeBlock = botctx.CodeBlock
	// Strike returns plain text without WhatsApp strikethrough formatting symbols (~).
	Strike = botctx.Strike
	// Strikef formats text according to format specifier without strikethrough formatting symbols.
	Strikef = botctx.Strikef
	// Quote returns plain text without WhatsApp quote formatting symbols (>).
	Quote = botctx.Quote
	// Quotef formats text according to format specifier without quote formatting symbols.
	Quotef = botctx.Quotef
	// NewText creates an interactive text builder instance.
	NewText = botctx.NewText
	// Sprintf returns a formatted string.
	Sprintf = botctx.Sprintf
	// CancelLoader is a no-op loader cancellation function.
	CancelLoader = botctx.CancelLoader
)
