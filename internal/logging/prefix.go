package logging

import (
	"io"
)

type prefixWriter struct {
	w         io.Writer
	prefix    string
	prefixLen int
}

func WithPrefix(w io.Writer, prefix string) *prefixWriter {
	return &prefixWriter{w, prefix, len(prefix)}
}

func (p prefixWriter) Write(data []byte) (int, error) {
	if p.prefixLen == 0 {
		return p.w.Write(data)
	}
	// Pre-allocate buffer with known size
	buf := make([]byte, p.prefixLen+len(data))
	copy(buf, p.prefix)
	copy(buf[p.prefixLen:], data)
	return p.w.Write(buf)
}
