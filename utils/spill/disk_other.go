//go:build !darwin && !linux

package main

import "errors"

func diskUsage(path string) (free uint64, total uint64, err error) {
	return 0, 0, errors.New("disk usage not supported on this platform")
}
