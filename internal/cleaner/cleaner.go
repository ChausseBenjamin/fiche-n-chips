package cleaner

import (
	"context"
	"fmt"
	"log/slog"

	"github.com/ChausseBenjamin/fiche-n-chips/internal/logging"
	"github.com/ChausseBenjamin/fiche-n-chips/internal/slugmgr"
	"github.com/ChausseBenjamin/fiche-n-chips/internal/util"

	"github.com/robfig/cron/v3"
)

type Cleaner struct {
	mgr     *slugmgr.SlugMgr
	cfg     *util.ConfigStore
	crontab *cron.Cron
}

func New(cfg *util.ConfigStore) (Cleaner, error) {
	crontab := cron.New()
	_, err := crontab.AddFunc(cfg.CronSchedule, func() {
		c := Cleaner{&cfg.SlugMgr, cfg, crontab}
		c.Cleanup(context.Background())
	})
	if err != nil {
		return Cleaner{}, fmt.Errorf("failed to add cron job: %w", err)
	}
	return Cleaner{&cfg.SlugMgr, cfg, crontab}, nil
}

func (c Cleaner) Start(ctx context.Context) {
	slog.InfoContext(ctx, "Starting automatic cleanup")
	c.crontab.Start()
}

// meant to be called as goroutine on startup and via cron afterwards
func (c Cleaner) Cleanup(ctx context.Context) {
	slog.InfoContext(ctx, "Running")
	i := 0
	slugs := c.mgr.FromNewest()
	// Enforce size constraint
	for c.mgr.TotalSize() > c.cfg.MaxSize {
		if slug, ok := util.Pop(&slugs); ok {
			if err := c.mgr.Remove(slug); err != nil {
				slog.ErrorContext(ctx, "failure while deleting slug",
					"id", slug,
					logging.ErrKey, err,
				)
				break
			}
			i++
		} else {
			// non-OK pop -> empty list...
			break
		}
	}
	// Enforce lifetime/expiry constraint
	for slug, ok := util.Pop(&slugs); ok; {
		age, err := c.mgr.Age(slug)
		if err != nil {
			slog.ErrorContext(ctx, "failed retrieving creation date for slog",
				"id", slug,
				logging.ErrKey, err,
			)
			break
		}
		if age > c.cfg.MaxAge {
			if err := c.mgr.Remove(slug); err != nil {
				slog.ErrorContext(ctx, "failure while deleting slug",
					"id", slug,
					logging.ErrKey, err,
				)
				break
			}
		}
	}
}
