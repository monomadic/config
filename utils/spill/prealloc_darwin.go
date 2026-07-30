//go:build darwin

package main

import (
	"errors"
	"os"

	"golang.org/x/sys/unix"
)

// preallocate reserves the destination file's blocks up front. On APFS this
// keeps a multi-gigabyte video from being laid down in fragments and saves the
// metadata churn of extending the file on every write. It also turns "the
// drive filled up" into an error before the first byte is written rather than
// six gigabytes in — the only failure worth surfacing, since a filesystem that
// simply doesn't support F_PREALLOCATE is not a problem.
func preallocate(f *os.File, size int64) error {
	if size <= 0 {
		return nil
	}
	// Contiguous is the ideal layout; if the free space is too fragmented for
	// that, F_ALLOCATEALL takes it in pieces.
	store := unix.Fstore_t{
		Flags:   unix.F_ALLOCATECONTIG,
		Posmode: unix.F_PEOFPOSMODE,
		Offset:  0,
		Length:  size,
	}
	err := unix.FcntlFstore(f.Fd(), unix.F_PREALLOCATE, &store)
	if err != nil {
		store.Flags = unix.F_ALLOCATEALL
		err = unix.FcntlFstore(f.Fd(), unix.F_PREALLOCATE, &store)
	}
	if errors.Is(err, unix.ENOSPC) {
		return err
	}
	return nil
}
