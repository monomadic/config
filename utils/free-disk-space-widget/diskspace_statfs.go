//go:build darwin || linux

package main

import "golang.org/x/sys/unix"

func statfsDiskSpace() (freeBytes uint64, totalBytes uint64, err error) {
	var stat unix.Statfs_t
	if err := unix.Statfs("/", &stat); err != nil {
		return 0, 0, err
	}

	blockSize := uint64(stat.Bsize)
	return stat.Bavail * blockSize, stat.Blocks * blockSize, nil
}
