package main

import (
	"context"
	"errors"
	"fmt"
	"io"
	"os"
	"path/filepath"
	"sync"
	"sync/atomic"
	"time"
	"unsafe"

	"github.com/cespare/xxhash/v2"
)

// errNotInTarget marks a source file with no counterpart in the target.
var errNotInTarget = errors.New("missing from target")

const (
	copyBufSize = 8 << 20  // 8 MiB per buffer
	copyBufs    = 3        // one being filled, one in flight, one being written
	ioAlign     = 16 << 10 // buffer start alignment, comfortably above any device block
)

// chunk is one filled buffer handed from the reader to the writer. buf is
// always the full-capacity buffer so it can be recycled; n is what to write.
type chunk struct {
	buf []byte
	n   int
}

// alignedBuf returns an n-byte buffer whose first byte sits on an ioAlign
// boundary. With F_NOCACHE the kernel talks to the device directly, and an
// unaligned buffer makes it bounce the data through an intermediate copy
// first. Go's collector does not move heap objects, so the alignment holds.
func alignedBuf(n int) []byte {
	buf := make([]byte, n+ioAlign)
	if off := int(uintptr(unsafe.Pointer(&buf[0])) % ioAlign); off != 0 {
		buf = buf[ioAlign-off:]
	}
	return buf[:n]
}

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
//
// Reads and writes are pipelined across two goroutines. A serial loop with
// F_NOCACHE on both fds leaves each drive idle while the other one works, so
// throughput lands at the harmonic mean of the two rather than at the speed of
// the slower one — on a fast source and a slow target that gives away a third
// of the write bandwidth.
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

	if err := preallocate(tmp, size); err != nil {
		tmp.Close()
		return err
	}

	hasher := xxhash.New()

	// copyBufs buffers cycle between the two goroutines: free carries empties
	// back to the reader, filled carries full ones to the writer. Both are
	// buffered to the full count, so neither send can ever block on the other
	// side being slow — only on there being no work.
	free := make(chan []byte, copyBufs)
	for i := 0; i < copyBufs; i++ {
		free <- alignedBuf(copyBufSize)
	}
	filled := make(chan chunk, copyBufs)
	stop := make(chan struct{})
	defer close(stop) // releases the reader when the writer leaves early

	// The reader also hashes, so the checksum costs nothing beyond the write
	// it overlaps with. Sequential reads keep the hash in stream order.
	var readErr error
	go func() {
		defer close(filled)
		for {
			var buf []byte
			select {
			case buf = <-free:
			case <-stop:
				return
			case <-ctx.Done():
				return
			}
			// ReadFull keeps every write device-block sized; a short read only
			// happens at EOF.
			n, rerr := io.ReadFull(src, buf)
			if n > 0 {
				if mode == verifyHash {
					_, _ = hasher.Write(buf[:n])
				}
				select {
				case filled <- chunk{buf: buf, n: n}:
				case <-stop:
					return
				case <-ctx.Done():
					return
				}
			}
			if rerr != nil {
				if rerr != io.EOF && rerr != io.ErrUnexpectedEOF {
					readErr = rerr
				}
				return
			}
		}
	}()

	var copied int64
	for c := range filled {
		if ctx.Err() != nil {
			tmp.Close()
			return ctx.Err()
		}
		nw, werr := tmp.Write(c.buf[:c.n])
		copied += int64(nw)
		if onProgress != nil {
			onProgress(copied)
		}
		if werr != nil {
			tmp.Close()
			return werr // typically ENOSPC
		}
		if nw < c.n {
			tmp.Close()
			return io.ErrShortWrite
		}
		free <- c.buf
	}
	// filled is closed, so the reader has returned and readErr is settled.
	if readErr != nil {
		tmp.Close()
		return readErr
	}
	if ctx.Err() != nil {
		tmp.Close()
		return ctx.Err()
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
		got, verr := hashFile(ctx, destPath, nil)
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

// verifyExisting checks a file already sitting in the target against its
// source, writing nothing. Size mode compares lengths; hash mode reads both
// files — concurrently, since they normally live on different drives — and
// compares digests. onProgress reports bytes read, out of the total returned
// by verifyTotal.
func verifyExisting(ctx context.Context, srcPath, destPath string, size int64, mode verifyMode, onProgress func(int64)) error {
	info, err := os.Stat(destPath)
	if err != nil {
		if os.IsNotExist(err) {
			return errNotInTarget
		}
		return err
	}
	if !info.Mode().IsRegular() {
		return errors.New("target is not a regular file")
	}
	if info.Size() != size {
		return fmt.Errorf("size mismatch: source %s, target %s",
			humanBytes(size), humanBytes(info.Size()))
	}
	if mode != verifyHash {
		if onProgress != nil {
			onProgress(size)
		}
		return nil
	}

	// Both readers share one counter and one reporter; the mutex keeps the
	// engine's progress bookkeeping single-threaded.
	var read atomic.Int64
	var mu sync.Mutex
	bump := func(n int64) {
		total := read.Add(n)
		if onProgress == nil {
			return
		}
		mu.Lock()
		onProgress(total)
		mu.Unlock()
	}

	ctx, cancel := context.WithCancel(ctx)
	defer cancel()

	type result struct {
		i   int
		sum uint64
		err error
	}
	res := make(chan result, 2)
	for i, p := range []string{srcPath, destPath} {
		go func(i int, path string) {
			sum, err := hashFile(ctx, path, bump)
			res <- result{i: i, sum: sum, err: err}
		}(i, p)
	}

	var sums [2]uint64
	var firstErr error
	for i := 0; i < 2; i++ {
		r := <-res
		if r.err != nil && firstErr == nil {
			firstErr = r.err
			cancel() // stop the other side early
		}
		sums[r.i] = r.sum
	}
	if firstErr != nil {
		return firstErr
	}
	if sums[0] != sums[1] {
		return fmt.Errorf("hash mismatch: source %016x, target %016x", sums[0], sums[1])
	}
	return nil
}

// verifyTotal is the number of bytes a verification of size bytes will read,
// so progress can be scaled against it.
func verifyTotal(size int64, mode verifyMode) int64 {
	if mode == verifyHash {
		return size * 2 // source and target are both read
	}
	return size
}

// hashFile reads a file back from disk and returns its xxhash64, so hash
// verification confirms what actually landed on the target rather than
// trusting the bytes we just streamed. onRead, when non-nil, is called with
// each chunk's byte count as it is consumed.
func hashFile(ctx context.Context, path string, onRead func(int64)) (uint64, error) {
	f, err := os.Open(path)
	if err != nil {
		return 0, err
	}
	defer f.Close()
	disableCache(f)

	h := xxhash.New()
	buf := alignedBuf(copyBufSize)
	for {
		select {
		case <-ctx.Done():
			return 0, ctx.Err()
		default:
		}
		n, rerr := f.Read(buf)
		if n > 0 {
			_, _ = h.Write(buf[:n])
			if onRead != nil {
				onRead(int64(n))
			}
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
