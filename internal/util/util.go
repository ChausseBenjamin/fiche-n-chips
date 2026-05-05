package util

import (
	"github.com/ChausseBenjamin/fiche-n-chips/internal/slugmgr"
	"net/http"
	"time"
)

type CtxKey int

const (
	ReqIDKey CtxKey = iota
	LoggerKey
)

// Any parameter/wrapper used by more than one module after initial startup
type ConfigStore struct {
	Port          int
	Addr          string
	MaxAge        time.Duration
	MaxSize       int // bytes
	CronSchedule  string
	HostingDir    string
	SlugMgr       slugmgr.SlugMgr
	AdminEmail    string
	DisplayDomain string
	UserDelPeriod time.Duration
	WithProfiling bool
}

func ClientIP(r *http.Request) string {
	xff := r.Header.Get("X-Forwarded-For")
	if xff != "" {
		// first IP in the chain
		for i := 0; i < len(xff); i++ {
			if xff[i] == ',' {
				return xff[:i]
			}
		}
		return xff
	}
	return r.RemoteAddr
}
func IsEmpty[T any](s []T) bool {
	return len(s) == 0
}
func Pop[T any](s *[]T) (T, bool) {
	if len(*s) == 0 {
		var zero T
		return zero, false
	}
	i := len(*s) - 1
	last := (*s)[i]
	var zero T
	(*s)[i] = zero
	*s = (*s)[:i]
	return last, true
}
