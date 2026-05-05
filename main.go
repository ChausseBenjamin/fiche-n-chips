package main

import (
	"context"
	"log/slog"
	"os"

	"github.com/ChausseBenjamin/fiche-n-chips/internal/app"
	"github.com/ChausseBenjamin/fiche-n-chips/internal/logging"
)

func main() {
	cmd := app.Command()

	if err := cmd.Run(context.Background(), os.Args); err != nil {
		slog.Error("Program quit unexpectedly", logging.ErrKey, err)
		os.Exit(1)
	}
}
