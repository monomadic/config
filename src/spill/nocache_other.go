//go:build !darwin

package main

import "os"

// disableCache is a no-op where F_NOCACHE has no equivalent.
func disableCache(f *os.File) {}
