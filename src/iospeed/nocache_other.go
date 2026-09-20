//go:build !darwin

package main

import "os"

// disableCache is a no-op on this platform; results may be inflated by the
// OS page cache (on Linux, consider dropping caches or using O_DIRECT).
func disableCache(f *os.File) error { return nil }
