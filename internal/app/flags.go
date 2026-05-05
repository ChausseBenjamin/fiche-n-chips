package app

import (
	"context"
	"errors"
	"fmt"
	"log/slog"
	"os"
	"regexp"
	"strings"
	"time"

	"github.com/ChausseBenjamin/fiche-n-chips/internal/logging"
	"github.com/robfig/cron/v3"
	"github.com/urfave/cli/v3"
)

const (
	FlagLogFormat = "log-format"
	FlagLogLevel  = "log-level"
	FlagLogOutput = "log-output"
	FlagLogTrace  = "log-trace"

	FlagAdminEmail      = "admin-email"
	FlagDisplayDomain   = "display-domain"
	FlagCleanupSchedule = "cleanup-schedule"
	FlagSlugCapacity    = "storage-capacity"
	FlagSlugExpiry      = "slug-expiry"
	FlagUserDeleteDelay = "user-deletion-period"
	FlagEnableProfiling = "enable-profiling"

	FlagGraceTimeout = "grace-timeout"
	FlagHostingDir   = "http-root-dir"
	FlagListenAddr   = "address"
	FlagListenPort   = "port"

	CatHTTP    = "http"
	CatLogging = "logging"
	CatMgmt    = "management"

	GB = 1 << 30
)

var (
	ErrOutOfBoundsPort error = errors.New("port is out of bounds")
	ErrNotDir          error = errors.New("provided path is not a directory")
)

func flags() []cli.Flag {
	return []cli.Flag{
		// General http settings
		&cli.StringFlag{
			Name:     FlagHostingDir,
			Category: CatHTTP,
			Value:    "/var/www",
			Usage:    "directory to host publicly",
			Sources:  cli.EnvVars("HTTP_ROOT_DIR"),
			Action:   validatePath,
		},
		&cli.IntFlag{
			Name:     FlagListenPort,
			Category: CatHTTP,
			Value:    8080,
			Sources:  cli.EnvVars("LISTEN_PORT"),
			Action:   validateListenPort,
		},
		&cli.StringFlag{
			Name:     FlagListenAddr,
			Category: CatHTTP,
			Value:    "0.0.0.0",
			Sources:  cli.EnvVars("LISTEN_ADDRESS"),
		},
		&cli.DurationFlag{
			Name:     FlagGraceTimeout,
			Category: CatHTTP,
			Value:    3 * time.Second,
			Sources:  cli.EnvVars("GRACEFUL_TIMEOUT"),
		}, // }}}
		// Management {{{
		&cli.IntFlag{
			Name:     FlagSlugExpiry,
			Category: CatMgmt,
			Value:    365,
			Usage:    "maximum storage duration for an entry (days)",
			Sources:  cli.EnvVars("SLUG_EXPIRY"),
		},
		&cli.IntFlag{
			Name:     FlagSlugCapacity,
			Category: CatMgmt,
			Value:    4 * GB,
			Usage:    "maximum capacity for the entire pastebin (all entries) in bytes",
			Sources:  cli.EnvVars("STORAGE_CAPACITY"),
		},
		&cli.StringFlag{
			Name:     FlagCleanupSchedule,
			Category: CatMgmt,
			Usage:    "cron schedule for the cleanup tasks",
			Value:    "0 * * * *",
			Sources:  cli.EnvVars("CLEANUP_SCHEDULE"),
			Action:   validateCron,
		},
		&cli.DurationFlag{
			Name:     FlagUserDeleteDelay,
			Category: CatMgmt,
			Usage:    "period during which users can delete an upload with a POST",
			Value:    15 * time.Minute,
			Sources:  cli.EnvVars("USER_DELETION_PERIOD"),
		},
		&cli.StringFlag{
			Name:     FlagAdminEmail,
			Category: CatMgmt,
			Usage:    "support email to request manual deletion",
			Value:    "benjamin@chausse.xyz",
			Sources:  cli.EnvVars("ADMIN_EMAIL"),
			Action:   validateEmail,
		},
		&cli.BoolFlag{
			Name:     FlagEnableProfiling,
			Category: CatMgmt,
			Usage:    "enable the profiling endpoint",
			Sources:  cli.EnvVars("ENABLE_PROFILING"),
		},
		&cli.StringFlag{
			Name:     FlagDisplayDomain,
			Category: CatMgmt,
			Usage:    "domain displayed in certain client responses",
			Value:    "https://1yr.ca",
			Sources:  cli.EnvVars("DISPLAY_DOMAIN"),
		}, // }}}
		// Logging {{{
		&cli.StringFlag{
			Name:     FlagLogLevel,
			Category: CatLogging,
			Value:    "info",
			Usage:    "debug, info, warn, error",
			Sources:  cli.EnvVars("LOG_LEVEL"),
			Action:   validateLogLevel,
		},
		&cli.StringFlag{
			Name:     FlagLogOutput,
			Category: CatLogging,
			Value:    "stdout",
			Usage:    "stdout, stderr, file",
			Sources:  cli.EnvVars("LOG_OUTPUT"),
			Action:   validateLogOutput,
		},
		&cli.StringFlag{
			Name:     FlagLogFormat,
			Category: CatLogging,
			Value:    "plain",
			Usage:    "plain, json, none",
			Sources:  cli.EnvVars("LOG_FORMAT"),
			Action:   validateLogFormat,
		},
		&cli.BoolFlag{
			Name:     FlagLogTrace,
			Category: CatLogging,
			Usage:    "enable trace (filename:line) within logs",
			Sources:  cli.EnvVars("LOG_TRACE"),
		}, // }}}
	}
}

func validateLogOutput(ctx context.Context, cmd *cli.Command, s string) error {
	switch s {
	case "stdout", "stderr":
		return nil
	default:
		// assume file
		f, err := os.OpenFile(s, os.O_CREATE|os.O_WRONLY|os.O_APPEND, 0644)
		if err != nil {
			slog.ErrorContext(
				ctx,
				fmt.Sprintf("Error creating/accessing provided log file %s", s),
			)
			return err
		}
		defer f.Close() //nolint:errcheck
		return nil
	}
}

func validateLogLevel(ctx context.Context, cmd *cli.Command, s string) error {
	for _, lvl := range []string{"deb", "inf", "warn", "err"} {
		if strings.Contains(strings.ToLower(s), lvl) {
			return nil
		}
	}
	slog.ErrorContext(
		ctx,
		fmt.Sprintf("Unknown log level provided: %s", s),
	)
	return logging.ErrInvalidLevel
}

func validateLogFormat(ctx context.Context, cmd *cli.Command, s string) error {
	s = strings.ToLower(s)
	if s == "json" || s == "plain" || s == "none" {
		return nil
	}
	return nil
}

func validateListenPort(ctx context.Context, cmd *cli.Command, p int) error {
	if p < 1024 || p > 65535 {
		slog.ErrorContext(
			ctx,
			fmt.Sprintf("Out-of-bound port provided: %d", p),
		)
		return ErrOutOfBoundsPort
	}
	return nil
}

func validatePath(ctx context.Context, cmd *cli.Command, path string) error {
	var (
		err  error = nil
		info os.FileInfo
	)
	info, err = os.Stat(path)
	if err == nil {
		if !info.IsDir() {
			err = ErrNotDir
		}
	}
	return err
}

func validateCron(ctx context.Context, cmd *cli.Command, schedule string) error {
	_, err := cron.ParseStandard(schedule)
	if err != nil {
		slog.ErrorContext(ctx, fmt.Sprintf("Invalid cron schedule: %s", schedule))
		return err
	}
	return nil
}

func validateEmail(ctx context.Context, cmd *cli.Command, email string) error {
	emailRegex := regexp.MustCompile(`^[a-zA-Z0-9._%+\-]+@[a-zA-Z0-9.\-]+\.[a-zA-Z]{2,}$`)
	if !emailRegex.MatchString(email) {
		slog.ErrorContext(ctx, fmt.Sprintf("Invalid email address: %s", email))
		return errors.New("invalid email format")
	}
	return nil
}
