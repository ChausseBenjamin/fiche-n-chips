package app

import (
	"context"
	"fmt"
	"log/slog"
	"net"
	"net/http"
	"os"
	"os/signal"
	"sync"
	"syscall"
	"time"

	"github.com/ChausseBenjamin/fiche-n-chips/internal/cleaner"
	"github.com/ChausseBenjamin/fiche-n-chips/internal/logging"
	"github.com/ChausseBenjamin/fiche-n-chips/internal/server"
	"github.com/ChausseBenjamin/fiche-n-chips/internal/slugmgr"
	"github.com/ChausseBenjamin/fiche-n-chips/internal/util"
	"github.com/urfave/cli/v3"
)

func action(ctx context.Context, cmd *cli.Command) error {
	err := logging.Setup(
		cmd.String(FlagLogLevel),
		cmd.String(FlagLogFormat),
		cmd.String(FlagLogOutput),
		cmd.Bool(FlagLogTrace),
	)
	if err != nil {
		slog.WarnContext(ctx, "Error(s) occurred during logger initialization",
			logging.ErrKey, err,
		)
	}
	slog.InfoContext(ctx, "Starting rafta server")

	errAppChan := make(chan error)
	shutdownDone := make(chan struct{}) // Signals when graceful shutdown is done

	var (
		once             sync.Once
		srv              http.Server
		gracefulShutdown func()
		brutalShutdown   func()
	)

	application := func() {
		handler, err := initApp(ctx, cmd)
		if err != nil {
			errAppChan <- err
			return
		}
		srv.Handler = handler

		//nolint:errcheck
		gracefulShutdown = func() {
			once.Do(func() { // Ensure brutal shutdown isn't triggered later
				srv.Shutdown(ctx)
				slog.InfoContext(ctx, "Application shutdown")
				close(shutdownDone) // Signal that graceful shutdown is complete
			})
		}

		//nolint:errcheck
		brutalShutdown = func() {
			slog.WarnContext(ctx,
				"Graceful shutdown delay exceeded, shutting down NOW!",
			)
			srv.Close()
		}

		port := fmt.Sprintf("%s:%d", cmd.String(FlagListenAddr), cmd.Int(FlagListenPort))
		listener, err := net.Listen("tcp", port)
		if err != nil {
			errAppChan <- err
			return
		}
		slog.InfoContext(ctx, "Server listening", "port", cmd.Int(FlagListenPort))

		if err := srv.Serve(listener); err != nil {
			errAppChan <- err
		}
	}
	go application()

	stopChan := waitForTermChan()
	running := true
	for running {
		select {
		case errApp := <-errAppChan:
			if errApp != nil {
				slog.ErrorContext(ctx, "Application error", logging.ErrKey, errApp)
			}
			return errApp
		case <-stopChan:
			slog.InfoContext(ctx, "Shutdown requested")
			go gracefulShutdown()

			select {
			case <-time.After(cmd.Duration(FlagGraceTimeout)): // Timeout exceeded
				brutalShutdown()
			case <-shutdownDone: // If graceful shutdown is timely, exit normally
			}
			running = false
		}
	}
	return nil
}

func waitForTermChan() chan os.Signal {
	stopChan := make(chan os.Signal, 1)
	signal.Notify(stopChan, syscall.SIGINT, syscall.SIGTERM)
	return stopChan
}

func initApp(ctx context.Context, cmd *cli.Command) (http.Handler, error) {
	// Setup common sanitized config for every module
	cfg := util.ConfigStore{
		Port:          cmd.Int(FlagListenPort),
		Addr:          cmd.String(FlagListenAddr),
		MaxAge:        time.Duration(24*cmd.Int(FlagSlugExpiry)) * time.Hour,
		MaxSize:       cmd.Int(FlagSlugCapacity),
		CronSchedule:  cmd.String(FlagCleanupSchedule),
		HostingDir:    cmd.String(FlagHostingDir),
		SlugMgr:       slugmgr.New(cmd.String(FlagHostingDir)),
		AdminEmail:    cmd.String(FlagAdminEmail),
		DisplayDomain: cmd.String(FlagDisplayDomain),
		UserDelPeriod: cmd.Duration(FlagUserDeleteDelay),
		WithProfiling: cmd.Bool(FlagEnableProfiling),
	}

	cleaner, err := cleaner.New(&cfg)
	if err != nil {
		return nil, fmt.Errorf("failed to initialize cleaner: %w", err)
	}
	cleaner.Start(ctx)

	handler := server.New(ctx, &cfg)
	return handler, nil
}
