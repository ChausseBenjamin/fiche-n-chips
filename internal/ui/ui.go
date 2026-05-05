package ui

import (
	"fmt"
	"net/http"
	"strings"
)

type attr string
type seq []attr

const (
	Reset attr = "0"

	Bold             attr = "1"
	BoldDel          attr = "21"
	Faint            attr = "2"
	FaintDel         attr = "22"
	Strikethrough    attr = "9"
	StrikethroughDel attr = "29"
	Italic           attr = "3"
	ItalicDel        attr = "23"
	Underline        attr = "4"
	UnderlineDel     attr = "24"

	SlowBlink  attr = "5"
	BlinkDel   attr = "25"
	RapidBlink attr = "6"
	Reverse    attr = "7"
	ReverseDel attr = "27"
	Conceal    attr = "8"
	ConcealDel attr = "28"

	BlackFg    attr = "30"
	BlackBg    attr = "40"
	RedFg      attr = "31"
	RedBg      attr = "41"
	GreenFg    attr = "32"
	GreenBg    attr = "42"
	YellowFg   attr = "33"
	YellowBg   attr = "43"
	BlueFg     attr = "34"
	BlueBg     attr = "44"
	MagentaFg  attr = "35"
	MagentaBg  attr = "45"
	CyanFg     attr = "36"
	CyanBg     attr = "46"
	WhiteFg    attr = "37"
	WhiteBg    attr = "47"
	ExtendedFg attr = "38"
	DefaultFg  attr = "39"

	escStart = "\x1b["
	esc      = "\x1b[m"
	escStop  = 'm'
	sep      = ';'
)

func (s seq) String() string {
	z := len(s)
	if z == 0 {
		return esc
	}
	var b strings.Builder

	n := (z << 2) + 2 // worst case size
	for _, a := range s {
		n += len(a)
	}
	b.Grow(n)

	b.WriteString(escStart)
	b.WriteString(string(s[0]))
	for i := 1; i < z; i++ {
		b.WriteByte(sep)
		b.WriteString(string(s[i]))
	}
	b.WriteByte(escStop)

	return b.String()
}

func String(inner any, s ...attr) uiString { return uiString{inner, s} }

type uiString struct {
	inner  any
	styles seq
}

func (s uiString) Raw() string { return fmt.Sprintf("%v", s.inner) }

func (s uiString) Flatten() string { return s.Raw() }

func (s uiString) String() string {
	var b strings.Builder

	z := len(s.styles)
	// best guess to total size as string size is unknowable at this point
	// I assume a fixed inner of 10 bytes, a worst case for the styles, and
	// the final reset takes three bytes
	b.Grow((z << 2) + 15)
	if z > 0 {
		b.WriteString(s.styles.String())
	}

	switch v := s.inner.(type) {
	case string:
		b.WriteString(v)
	case fmt.Stringer:
		b.WriteString(v.String())
	default:
		fmt.Fprintf(&b, "%v", v)
	}
	b.WriteString(esc)
	return b.String()
}

// Sprintf wrapper that manages whether or not text
func Output(w http.ResponseWriter, r *http.Request, t string, args ...any) (int, error) {
	plainHeader := strings.TrimSpace(strings.ToLower(r.Header.Get("X-Disable-Colors")))
	// user went out of his way to set the flag,
	// he most likely wants it disabled
	plain := len(plainHeader) > 0 ||
		// accepting html hints at the request coming from a browser
		strings.Contains(r.Header.Get("Accept"), "html")

	// To set this correctly, the user most likely knows what he's doing and
	// wants to override the current browser protection.
	if plainHeader == "false" {
		plain = false
	}

	if plain {
		for i, v := range args {
			if str, ok := v.(uiString); ok {
				args[i] = str.Flatten()
			}
		}
	}
	msg := fmt.Sprintf(t, args...)
	return w.Write([]byte(msg))
}
