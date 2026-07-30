//go:build !darwin

package main

import "os"

// preallocate is a no-op where F_PREALLOCATE has no equivalent.
func preallocate(f *os.File, size int64) error { return nil }
