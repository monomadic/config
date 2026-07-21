//go:build !darwin

package main

import "errors"

// perCoreUsage is macOS-only; the Mach host_processor_info path has no portable
// equivalent here. The widget still builds on other platforms for tooling's sake.
func perCoreUsage() ([]float64, error) {
	return nil, errors.New("per-core CPU usage is only supported on darwin")
}
