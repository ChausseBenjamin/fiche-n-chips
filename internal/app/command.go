package app

import (
	"github.com/urfave/cli/v3"
)

const (
	AppName  = "fiche-n-chips"
	AppUsage = "The fiche pastebin, with extra snacks"
)

var version = "COMPILED"

func Command() *cli.Command {
	return &cli.Command{
		Name:    AppName,
		Usage:   AppUsage,
		Authors: []any{"Benjamin Chausse <benjamin@chausse.xyz>"},
		Version: version,
		Flags:   flags(),
		Action:  action,
	}
}
