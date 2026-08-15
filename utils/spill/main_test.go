package main

import "testing"

// A move built on spill deletes its sources when spill exits 0, so these cases
// are the difference between "the copy is really there" and data loss.
func TestExitCode(t *testing.T) {
	cases := []struct {
		name     string
		sum      summary
		reported bool
		uiFailed int
		want     int
	}{
		{"clean run", summary{copied: 3}, true, 0, 0},
		{"nothing to do", summary{}, true, 0, 0},
		{"skips are not failures", summary{copied: 1, skipped: 2}, true, 0, 0},
		{"filtered out by strategy", summary{filtered: 4}, true, 0, 0},
		{"drive full is a real stop", summary{copied: 2, stoppedFull: true}, true, 0, 0},
		{"a failed copy", summary{copied: 2, failed: 1}, true, 0, 1},
		{"cancelled mid-run", summary{copied: 1, cancelled: true}, true, 0, 1},
		{"cancelled having copied nothing", summary{cancelled: true}, true, 0, 1},
		{"UI quit before the engine reported", summary{}, false, 0, 1},
		{"failure seen only by the UI", summary{}, true, 1, 1},
	}

	for _, c := range cases {
		t.Run(c.name, func(t *testing.T) {
			if got := exitCode(c.sum, c.reported, c.uiFailed); got != c.want {
				t.Errorf("exitCode(%+v, reported=%v, uiFailed=%d) = %d, want %d",
					c.sum, c.reported, c.uiFailed, got, c.want)
			}
		})
	}
}
