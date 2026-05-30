//go:build linux

package main

func diskSpace() (freeBytes uint64, totalBytes uint64, err error) {
	return statfsDiskSpace()
}
