//go:build darwin || linux

package main

import "golang.org/x/sys/unix"

// diskUsage returns the free (available to an unprivileged process) and total
// bytes of the filesystem that contains path.
func diskUsage(path string) (free uint64, total uint64, err error) {
	var st unix.Statfs_t
	if err := unix.Statfs(path, &st); err != nil {
		return 0, 0, err
	}
	bs := uint64(st.Bsize)
	return st.Bavail * bs, st.Blocks * bs, nil
}
