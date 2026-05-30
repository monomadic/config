//go:build darwin

package main

/*
#cgo darwin CFLAGS: -x objective-c -fobjc-arc
#cgo darwin LDFLAGS: -framework Foundation
#import <Foundation/Foundation.h>
#include <stdint.h>
#include <stdlib.h>

static int volumeCapacityForPath(const char *path, uint64_t *availableBytes, uint64_t *totalBytes) {
	@autoreleasepool {
		NSString *pathString = [NSString stringWithUTF8String:path];
		if (pathString == nil) {
			return 0;
		}

		NSURL *url = [NSURL fileURLWithPath:pathString isDirectory:YES];
		NSNumber *available = nil;
		if (![url getResourceValue:&available forKey:NSURLVolumeAvailableCapacityForImportantUsageKey error:nil] || available == nil) {
			return 0;
		}

		NSNumber *total = nil;
		if (![url getResourceValue:&total forKey:NSURLVolumeTotalCapacityKey error:nil] || total == nil) {
			return 0;
		}

		*availableBytes = [available unsignedLongLongValue];
		*totalBytes = [total unsignedLongLongValue];
		return 1;
	}
}
*/
import "C"

import "unsafe"

func diskSpace() (freeBytes uint64, totalBytes uint64, err error) {
	path := C.CString("/")
	defer C.free(unsafe.Pointer(path))

	var available C.uint64_t
	var total C.uint64_t
	if C.volumeCapacityForPath(path, &available, &total) == 0 {
		return statfsDiskSpace()
	}

	return uint64(available), uint64(total), nil
}
