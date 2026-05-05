package server

import (
	"context"
	_ "embed"
	"log/slog"
	"net/http"
	"net/http/pprof"
	"os"
	"path/filepath"
	"strings"

	"github.com/google/uuid"

	"github.com/ChausseBenjamin/fiche-n-chips/internal/logging"
	"github.com/ChausseBenjamin/fiche-n-chips/internal/ui"
	"github.com/ChausseBenjamin/fiche-n-chips/internal/util"
)

const (
	keySlug = "slug"
)

var (
	//go:embed index.html
	embeddedHome []byte
	//go:embed 404.html
	embedded404 []byte
	sorry       = ui.String("Sorry Eh...", ui.RedFg, ui.Bold)
)

const tmpltDelFailure = `You are trying to delete '%s' but it has been created more than %v
ago and can no longer be deleted automatically (created %v ago).

If you still wish to delete the contents of this file, you may email me
at %v to have this content removed manually.
%s
`
const robotsTxt = `
User-agent: *
Disallow: /

Allow: /index.html
Allow: /404.html
`

// Barebones for now but I might want to embed
// templating in the future...
type webpage struct {
	contents []byte
}

func New(ctx context.Context, cfg *util.ConfigStore) http.Handler {
	var (
		mux         = http.NewServeMux()
		home        = newPage(ctx, cfg.HostingDir, "index.html", embeddedHome)
		lost        = newPage(ctx, cfg.HostingDir, "404.html", embedded404)
		robots      = newPage(ctx, cfg.HostingDir, "robotsTxt", []byte(robotsTxt))
		lostHandler = handlePage(lost, "404")
	)

	if cfg.WithProfiling {
		log := logging.TryFromContext(ctx, "profiling-creation")
		log.InfoContext(ctx, "profiling started")
		mux.HandleFunc("GET /debug/pprof/", pprof.Index)
		mux.HandleFunc("GET /debug/pprof/{profile}", pprof.Index)
		mux.HandleFunc("GET /debug/pprof/cmdline", pprof.Cmdline)
		mux.HandleFunc("GET /debug/pprof/profile", pprof.Profile)
		mux.HandleFunc("GET /debug/pprof/symbol", pprof.Symbol)
		mux.HandleFunc("GET /debug/pprof/trace", pprof.Trace)
	}

	for _, path := range []string{"GET /", "GET /index.html"} {
		mux.HandleFunc(path, handlePage(home, "homepage"))
	}

	for _, path := range []string{"GET /404", "GET /404.html"} {
		mux.HandleFunc(path, lostHandler)
	}

	for _, path := range []string{"GET /{slug}", "GET /{slug}/", "GET /{slug}/index.txt"} {
		mux.HandleFunc(path, handleSlug(cfg, lostHandler))
	}

	mux.HandleFunc("GET /robots.txt", handlePage(robots, "robots.txt"))
	mux.HandleFunc("POST /delete", handleDelete(cfg))

	return withRequestID(mux)
}

func withRequestID(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		id := uuid.New().String()
		ctx := context.WithValue(r.Context(), util.ReqIDKey, id)
		w.Header().Add("X-Request-ID", id)
		logger := logging.WithRequestInfo(slog.Default(), r)
		ctx = context.WithValue(ctx, util.LoggerKey, logger)
		r = r.WithContext(ctx)
		next.ServeHTTP(w, r)
	})
}

func newPage(ctx context.Context, root string, filename string, data []byte) webpage {
	log := logging.TryFromContext(ctx, "create_webpage",
		slog.String("webpage", filename),
	)
	var page webpage
	// override embedded homepage if `/index.html` exists
	customPath := filepath.Join(root, filename)
	if info, err := os.Stat(customPath); err == nil {
		log.DebugContext(ctx,
			"detected custom webpage, attempting to use it",
			"webpage", filename,
		)
		if !info.IsDir() {
			var err error
			buf, err := os.ReadFile(customPath)
			if err != nil {
				log.ErrorContext(ctx, "failed to read custom homepage, reverting to default",
					logging.ErrKey, err,
				)
			} else {
				page.contents = buf
			}
		}
	}
	if util.IsEmpty(page.contents) {
		log.WarnContext(ctx,
			"custom webpage appears empty, falling back to default",
			"webpage", filename,
		)
		page.contents = data
	}
	return page
}

func handlePage(page webpage, name string) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		log := logging.TryFromRequest(r)

		if _, err := w.Write(page.contents); err != nil {
			log.ErrorContext(r.Context(),
				"failure sending webpage to client",
				"name", name,
				logging.ErrKey, err,
			)
		}
	}
}

func handleSlug(cfg *util.ConfigStore, lostHandler http.HandlerFunc) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		ctx := r.Context()
		log := logging.TryFromRequest(r)

		slug := r.PathValue(keySlug)
		if !cfg.SlugMgr.Exists(slug) {
			log.WarnContext(r.Context(),
				"user tried to read an empty page",
			)
			if accepts := r.Header.Get("Accept"); strings.Contains(accepts, "html") {
				lostHandler(w, r)
			} else {
				w.WriteHeader(http.StatusNotFound)
				if _, err := ui.Output(w, r, "You requested '%v' but it doesn't seem to exist.\n%s\n",
					ui.String(slug, ui.YellowFg, ui.Bold),
					sorry,
				); err != nil {
					log.ErrorContext(ctx, "failed to inform the user a slug doesn't exist")
				}
			}
			return
		}
		contents, err := cfg.SlugMgr.Get(slug)
		if err != nil {
			log.ErrorContext(ctx,
				"paste was found but failed to open",
				logging.ErrKey, err,
			)
			w.WriteHeader(http.StatusInternalServerError)
			if _, err := ui.Output(w, r, "An internal error occurred while retrieving '%s'.\n%s\n",
				ui.String(slug, ui.YellowFg, ui.Bold),
				sorry,
			); err != nil {
				log.ErrorContext(ctx, "failed to inform user of retrieval error",
					logging.ErrKey, err,
				)
			}
			return
		} else if _, err := w.Write([]byte(contents)); err != nil {
			log.ErrorContext(ctx,
				"failed to serve paste to client",
				logging.ErrKey, err,
			)
		}
	}
}

func handleDelete(cfg *util.ConfigStore) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		log := logging.TryFromRequest(r)
		ctx := r.Context()

		slug := r.URL.Query().Get(keySlug)
		if slug == "" {
			log.WarnContext(ctx,
				"user requested to delete an empty slug",
			)
			w.WriteHeader(http.StatusBadRequest)
			if _, err := ui.Output(w, r, "Bad request: When requesting a deletion you must provide the slug as a parameter...\nEx: `curl -X '%v/delete%v%v'\n%s\n",
				cfg.DisplayDomain,
				ui.String("?slug=", ui.YellowFg, ui.Bold),
				ui.String("abc123", ui.YellowFg, ui.Bold, ui.Underline),
				sorry,
			); err != nil {
				log.ErrorContext(ctx, "failed to send bad request response",
					logging.ErrKey, err,
				)
			}
			return
		} else if !cfg.SlugMgr.Exists(slug) {
			log.WarnContext(ctx,
				"user tried to delete an empty page",
			)
			w.WriteHeader(http.StatusNotFound)
			if _, err := ui.Output(w, r, "You requested to delete '%v' but it doesn't seem to exist.\n%s\n",
				ui.String(slug, ui.YellowFg, ui.Bold),
				sorry,
			); err != nil {
				log.ErrorContext(ctx, "failed to send not found response",
					logging.ErrKey, err,
				)
			}
			return
		}
		age, err := cfg.SlugMgr.Age(slug)
		if err != nil {
			log.ErrorContext(ctx,
				"failed to retrieve slug creation date",
				logging.ErrKey, err,
			)
			w.WriteHeader(http.StatusInternalServerError)
			if _, err := ui.Output(w, r, "An internal error occured while checking the creation date for '%s'.\nPlease contact %s to request a manual deletion.\n%s\n",
				ui.String(slug, ui.YellowFg, ui.Bold),
				ui.String(cfg.AdminEmail, ui.Underline),
				sorry,
			); err != nil {
				log.ErrorContext(ctx, "failed to send internal error response",
					logging.ErrKey, err,
				)
			}
			return
		}
		if age > cfg.UserDelPeriod {
			log.WarnContext(ctx,
				"user tried to delete file past its grace period",
			)
			w.WriteHeader(http.StatusForbidden)
			if _, err := ui.Output(w, r, tmpltDelFailure,
				ui.String(slug, ui.YellowFg, ui.Bold),
				cfg.UserDelPeriod,
				ui.String(age, ui.RedFg),
				ui.String(cfg.AdminEmail, ui.Underline),
				sorry,
			); err != nil {
				log.ErrorContext(ctx, "failure while sending response to user",
					logging.ErrKey, err,
				)
			}
		} else if err := cfg.SlugMgr.Remove(slug); err != nil {
			log.ErrorContext(ctx, "failed to delete slug",
				logging.ErrKey, err,
			)
			w.WriteHeader(http.StatusInternalServerError)
			if _, err := ui.Output(w, r,
				"An internal error occured trying to delete '%s'...\nPlease contact %s to request a manual deletion.\n%s\n",
				ui.String(slug, ui.YellowFg, ui.Bold),
				ui.String(cfg.AdminEmail, ui.Underline),
				sorry,
			); err != nil {
				log.ErrorContext(ctx, "failed telling user a failure occured",
					logging.ErrKey, err,
				)
			}
		} else {
			w.WriteHeader(http.StatusOK)
			if _, err := ui.Output(w, r, "Successfully deleted '%s', %s\n",
				ui.String(slug, ui.YellowFg, ui.Bold), ui.String("cheers!", ui.GreenFg),
			); err != nil {
				log.ErrorContext(ctx, "failed telling user deletion succeeded",
					logging.ErrKey, err,
				)
			}
		}
	}
}
