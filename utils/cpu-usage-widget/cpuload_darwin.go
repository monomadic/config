//go:build darwin

package main

/*
#cgo darwin CFLAGS: -x objective-c
#include <mach/mach.h>
#include <mach/processor_info.h>
#include <mach/mach_host.h>
#include <stdint.h>

// cpuTicks reads cumulative per-core CPU tick counters from the kernel.
// It fills the caller-provided arrays (up to maxCores) and returns the number
// of cores reported, or 0 on failure. This is a single Mach call — no process
// enumeration, no sampling window — so it costs microseconds.
static int cpuTicks(int maxCores, uint64_t *user, uint64_t *system, uint64_t *idle, uint64_t *nice) {
	natural_t cpuCount = 0;
	processor_info_array_t info = NULL;
	mach_msg_type_number_t infoCount = 0;

	kern_return_t kr = host_processor_info(mach_host_self(), PROCESSOR_CPU_LOAD_INFO, &cpuCount, &info, &infoCount);
	if (kr != KERN_SUCCESS || info == NULL) {
		return 0;
	}

	int n = (int)cpuCount;
	if (n > maxCores) {
		n = maxCores;
	}

	processor_cpu_load_info_t load = (processor_cpu_load_info_t)info;
	for (int i = 0; i < n; i++) {
		user[i] = load[i].cpu_ticks[CPU_STATE_USER];
		system[i] = load[i].cpu_ticks[CPU_STATE_SYSTEM];
		idle[i] = load[i].cpu_ticks[CPU_STATE_IDLE];
		nice[i] = load[i].cpu_ticks[CPU_STATE_NICE];
	}

	vm_deallocate(mach_task_self(), (vm_address_t)info, infoCount * sizeof(int));
	return n;
}
*/
import "C"

import (
	"errors"
	"sync"
)

const maxCores = 256

type coreTicks struct {
	user, system, idle, nice uint64
}

var (
	cpuMu     sync.Mutex
	prevTicks []coreTicks
)

func readCoreTicks() ([]coreTicks, error) {
	var (
		user   [maxCores]C.uint64_t
		system [maxCores]C.uint64_t
		idle   [maxCores]C.uint64_t
		nice   [maxCores]C.uint64_t
	)

	n := int(C.cpuTicks(C.int(maxCores), &user[0], &system[0], &idle[0], &nice[0]))
	if n <= 0 {
		return nil, errors.New("host_processor_info returned no cores")
	}

	ticks := make([]coreTicks, n)
	for i := 0; i < n; i++ {
		ticks[i] = coreTicks{
			user:   uint64(user[i]),
			system: uint64(system[i]),
			idle:   uint64(idle[i]),
			nice:   uint64(nice[i]),
		}
	}
	return ticks, nil
}

// perCoreUsage returns the busy ratio [0,1] for each core, measured over the
// interval since the previous call. The first call establishes a baseline and
// returns all zeros — CPU utilization is a rate, so it needs two samples.
func perCoreUsage() ([]float64, error) {
	current, err := readCoreTicks()
	if err != nil {
		return nil, err
	}

	cpuMu.Lock()
	defer cpuMu.Unlock()

	usage := make([]float64, len(current))
	if len(prevTicks) == len(current) {
		for i := range current {
			deltaBusy := (current[i].user - prevTicks[i].user) +
				(current[i].system - prevTicks[i].system) +
				(current[i].nice - prevTicks[i].nice)
			deltaTotal := deltaBusy + (current[i].idle - prevTicks[i].idle)
			if deltaTotal > 0 {
				usage[i] = float64(deltaBusy) / float64(deltaTotal)
			}
		}
	}

	prevTicks = current
	return usage, nil
}
