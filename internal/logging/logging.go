package logging

import (
	"context"
	"errors"
	"io"
	"log/slog"
	"net/http"
	"os"
	"strings"

	"github.com/ChausseBenjamin/fiche-n-chips/internal/util"
)

const (
	ErrKey   = "error_message"
	ReqIDKey = "request_id"
)

var (
	ErrInvalidLevel  = errors.New("invalid log level")
	ErrInvalidFormat = errors.New("invalid log format")
)

const DisableLogs = "disable"

func Setup(lvlStr, fmtStr, outStr string, withTrace bool) error {
	output, outputErr := setOutput(outStr)
	format, formatErr := setFormat(fmtStr)
	level, levelErr := setLevel(lvlStr)

	var h slog.Handler
	if format == DisableLogs {
		h = DiscardHandler{}
	} else {
		opts := &slog.HandlerOptions{
			Level:     level,
			AddSource: true,
		}
		if format == "json" {
			h = slog.NewJSONHandler(output, opts)
		} else {
			prefixStr := "Fiche N Chips 🍟"
			h = slog.NewTextHandler(WithPrefix(output, prefixStr), opts)
		}

		if withTrace {
			h = withStackTrace(h)
		}
		h = withTrackedContext(h, util.ReqIDKey, ReqIDKey)
	}

	slog.SetDefault(slog.New(h))
	return errors.Join(outputErr, formatErr, levelErr)
}

func WithRequestInfo(logger *slog.Logger, r *http.Request) *slog.Logger {
	return logger.With(
		slog.Group("client",
			"ip", util.ClientIP(r),
			"remote_addr", r.RemoteAddr,
			"user_agent", r.UserAgent(),
		),
		slog.Group("http",
			"method", r.Method,
			"pattern", r.Pattern,
			"path", r.URL.Path,
		),
	)
}

func TryFromRequest(r *http.Request) *slog.Logger {
	ctx := r.Context()
	if v := ctx.Value(util.LoggerKey); v != nil {
		if logger, ok := v.(*slog.Logger); ok {
			return logger
		}
	}
	// nolint:sloglint
	slog.WarnContext(ctx, "Logger not found in request context, using default logger",
		slog.String("path", r.URL.Path),
		slog.Group("request",
			slog.String("method", r.Method),
			slog.String("pattern", r.Pattern),
			slog.String("ip", util.ClientIP(r)),
			slog.String("remote_addr", r.RemoteAddr),
			slog.String("user_agent", r.UserAgent()),
		),
	)
	return slog.Default()
}

// Context is harder to pinpoint in a codebase on it's own as there
// is no garantee a key will be there and not request info to help poinpoint a
// source. The origin is used to alleviate that.
func TryFromContext(ctx context.Context, origin string, args ...any) *slog.Logger {
	if v := ctx.Value(util.LoggerKey); v != nil {
		if logger, ok := v.(*slog.Logger); ok {
			return logger
		}
	}

	fullargs := make([]any, len(args)+1)
	fullargs = append(fullargs, slog.String("origin", origin))
	fullargs = append(fullargs, args...)

	// nolint:sloglint
	slog.WarnContext(ctx, "Logger not found in context, using default logger",
		fullargs...,
	)
	return slog.Default()
}

func setLevel(target string) (slog.Level, error) {
	for _, l := range []struct {
		prefix string
		level  slog.Level
	}{
		{"deb", slog.LevelDebug},
		{"inf", slog.LevelInfo},
		{"warn", slog.LevelWarn},
		{"err", slog.LevelError},
	} {
		if strings.HasPrefix(strings.ToLower(target), l.prefix) {
			return l.level, nil
		}
	}
	return slog.LevelInfo, ErrInvalidLevel
}

func setFormat(f string) (string, error) {
	switch f {
	case "plain", "text":
		return "text", nil
	case "json", "structured":
		return "json", nil
	case "none", "off":
		return DisableLogs, nil
	}
	return "text", ErrInvalidFormat
}

func setOutput(path string) (io.Writer, error) {
	switch path {
	case "stdout":
		return os.Stdout, nil
	case "stderr":
		return os.Stderr, nil
	default:
		f, err := os.OpenFile(path, os.O_CREATE|os.O_WRONLY|os.O_APPEND, 0644)
		if err != nil {
			return os.Stdout, err
		} else {
			return f, nil
		}
	}
}
