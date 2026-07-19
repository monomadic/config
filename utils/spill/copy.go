package main

import (
	"context"
	"fmt"
	"io"
	"os"
	"path/filepath"
	"time"

	"github.com/cespare/xxhash/v2"
)

const copyBufSize = 4 << 20 // 4 MiB

// speedometer produces a smoothed bytes/second reading from successive
// (time, cumulative-bytes) samples via an exponential moving average.
type speedometer struct {
	ewma      float64
	lastTime  time.Time
	lastBytes int64
	started   bool
}

func (s *speedometer) reset() { *s = speedometer{} }

func (s *speedometer) sample(now time.Time, total int64) float64 {
	if !s.started {
		s.started = true
		s.lastTime = now
		s.lastBytes = total
		return 0
	}
	dt := now.Sub(s.lastTime).Seconds()
	if dt <= 0 {
		return s.ewma
	}
	inst := float64(total-s.lastBytes) / dt
	s.lastTime = now
	s.lastBytes = total
	const alpha = 0.3
	if s.ewma == 0 {
		s.ewma = inst
	} else {
		s.ewma = alpha*inst + (1-alpha)*s.ewma
	}
	return s.ewma
}

// copyOne copies srcPath to destPath through a temp file in the destination
// directory, then atomically renames it into place. It reports cumulative
// bytes via onProgress, honours ctx cancellation, and enforces the requested
// verification. A returned error leaves no partial file behind.
func copyOne(ctx context.Context, srcPath, destPath string, size int64, mode verifyMode, onProgress func(int64)) (err error) {
	src, err := os.Open(srcPath)
	if err != nil {
		return err
	}
	defer src.Close()
	disableCache(src)

	destDir := filepath.Dir(destPath)
	tmp, err := os.CreateTemp(destDir, ".spill-*.part")
	if err != nil {
		return err
	}
	tmpName := tmp.Name()
	disableCache(tmp)

	renamed := false
	defer func() {
		if !renamed {
			_ = os.Remove(tmpName)
		}
	}()

	hasher := xxhash.New()
	buf := make([]byte, copyBufSize)
	var copied int64

	for {
		select {
		case <-ctx.Done():
			tmp.Close()
			return ctx.Err()
		default:
		}

		nr, rerr := src.Read(buf)
		if nr > 0 {
			if mode == verifyHash {
				_, _ = hasher.Write(buf[:nr])
			}
			nw, werr := tmp.Write(buf[:nr])
			copied += int64(nw)
			if onProgress != nil {
				onProgress(copied)
			}
			if werr != nil {
				tmp.Close()
				return werr // typically ENOSPC
			}
			if nw < nr {
				tmp.Close()
				return io.ErrShortWrite
			}
		}
		if rerr == io.EOF {
			break
		}
		if rerr != nil {
			tmp.Close()
			return rerr
		}
	}

	if err := tmp.Sync(); err != nil {
		tmp.Close()
		return err
	}
	if err := tmp.Close(); err != nil {
		return err
	}

	if mode == verifySize && copied != size {
		return fmt.Errorf("size mismatch: wrote %d of %d bytes", copied, size)
	}

	if err := os.Rename(tmpName, destPath); err != nil {
		return err
	}
	renamed = true

	// Best-effort metadata preservation; failures here don't fail the copy.
	if info, statErr := os.Stat(srcPath); statErr == nil {
		_ = os.Chmod(destPath, info.Mode().Perm())
		_ = os.Chtimes(destPath, time.Now(), info.ModTime())
	}

	if mode == verifyHash {
		want := hasher.Sum64()
		got, verr := hashFile(ctx, destPath)
		if verr != nil {
			_ = os.Remove(destPath)
			return verr
		}
		if got != want {
			_ = os.Remove(destPath)
			return fmt.Errorf("hash mismatch: source %016x, copy %016x", want, got)
		}
	}

	return nil
}

// hashFile reads a file back from disk and returns its xxhash64, so hash
// verification confirms what actually landed on the target rather than
// trusting the bytes we just streamed.
func hashFile(ctx context.Context, path string) (uint64, error) {
	f, err := os.Open(path)
	if err != nil {
		return 0, err
	}
	defer f.Close()
	disableCache(f)

	h := xxhash.New()
	buf := make([]byte, copyBufSize)
	for {
		select {
		case <-ctx.Done():
			return 0, ctx.Err()
		default:
		}
		n, rerr := f.Read(buf)
		if n > 0 {
			_, _ = h.Write(buf[:n])
		}
		if rerr == io.EOF {
			break
		}
		if rerr != nil {
			return 0, rerr
		}
	}
	return h.Sum64(), nil
}
