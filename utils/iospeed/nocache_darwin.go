//go:build darwin

package main

import (
	"os"

	"golang.org/x/sys/unix"
)

// disableCache bypasses the page cache for this fd (F_NOCACHE), so reads
// and writes hit the disk instead of RAM.
func disableCache(f *os.File) error {
	_, err := unix.FcntlInt(f.Fd(), unix.F_NOCACHE, 1)
	return err
}
