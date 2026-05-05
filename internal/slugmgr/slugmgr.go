package slugmgr

import (
	"io/fs"
	"os"
	"path/filepath"
	"sort"
	"time"
)

type SlugMgr struct {
	root string
}

func New(root string) SlugMgr {
	return SlugMgr{root}
}

// list all directories in `{mgr.root}/` from newest
// to oldest (only the slug strings without the `/`)
//
// NOTE: FromNewest is prefered to FromOldest when deleting stuff since popping
// from a list is simpler that removing the first element and rebasing the
// rest...
func (mgr SlugMgr) FromNewest() []string {
	entries, err := os.ReadDir(mgr.root)
	if err != nil {
		return []string{}
	}

	var slugs []string
	var modTimes []time.Time

	for _, entry := range entries {
		if entry.IsDir() {
			info, err := entry.Info()
			if err != nil {
				continue
			}
			slugs = append(slugs, entry.Name())
			modTimes = append(modTimes, info.ModTime())
		}
	}

	// Sort by modification time, newest first
	sort.Slice(slugs, func(i, j int) bool {
		return modTimes[i].After(modTimes[j])
	})

	return slugs
}

// total size of the `{mgr.root}/` directory
func (mgr SlugMgr) TotalSize() int {
	var totalSize int64
	filepath.WalkDir(mgr.root, func(path string, d fs.DirEntry, err error) error {
		if err != nil {
			return nil // Skip errors
		}
		if !d.IsDir() {
			info, err := d.Info()
			if err != nil {
				return nil
			}
			totalSize += info.Size()
		}
		return nil
	})
	return int(totalSize)
}

// whether a slug directory exists
// - pattern: `{mgr.root}/{slug}/index.txt`
func (mgr SlugMgr) Exists(slug string) bool {
	indexPath := filepath.Join(mgr.root, slug, "index.txt")
	_, err := os.Stat(indexPath)
	return !os.IsNotExist(err)
}

// contents of `index.txt` inside the slug directory
// - pattern: `{mgr.root}/{slug}/index.txt`
func (mgr SlugMgr) Get(slug string) (string, error) {
	indexPath := filepath.Join(mgr.root, slug, "index.txt")
	content, err := os.ReadFile(indexPath)
	if err != nil {
		return "", err
	}
	return string(content), nil
}

// deletes the slug directory
// - pattern: `{mgr.root}/{slug}/`
func (mgr SlugMgr) Remove(slug string) error {
	slugDir := filepath.Join(mgr.root, slug)
	return os.RemoveAll(slugDir)
}

// how long ago a slug directory has been created
// - pattern: `{mgr.root}/{slug}/`
func (mgr SlugMgr) Age(slug string) (time.Duration, error) {
	slugDir := filepath.Join(mgr.root, slug)
	info, err := os.Stat(slugDir)
	if err != nil {
		return 0, err
	}
	return time.Since(info.ModTime()), nil
}
