//go:build darwin

package main

import (
	"os"

	"golang.org/x/sys/unix"
)

// disableCache asks the kernel to bypass the unified buffer cache for this fd
// (F_NOCACHE). Copying media by the gigabyte otherwise evicts everything else
// from RAM, and the page cache would also make the measured write speed a
// fiction — this keeps throughput honest and the machine responsive.
func disableCache(f *os.File) {
	_, _ = unix.FcntlInt(f.Fd(), unix.F_NOCACHE, 1)
}
