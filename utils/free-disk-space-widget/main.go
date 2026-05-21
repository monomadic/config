package main

import (
	"fmt"
	"math"
	"time"

	"github.com/getlantern/systray"
	"golang.org/x/sys/unix"
)

const (
	diskIcon    = "􀤂"
	lowDiskIcon = "􁘥"
)

func main() {
	systray.Run(onReady, onExit)
}

func formatBytesToGb(bytes int64) string {
	freeGB := int64(math.Round(float64(bytes) / 1024 / 1024 / 1024))
	return fmt.Sprintf("%d GB", freeGB)
}

func diskSpace() (freeBytes uint64, totalBytes uint64, err error) {
	var stat unix.Statfs_t
	if err := unix.Statfs("/", &stat); err != nil {
		return 0, 0, err
	}

	blockSize := uint64(stat.Bsize)
	return stat.Bavail * blockSize, stat.Blocks * blockSize, nil
}

func iconForFreeSpace(freeBytes, totalBytes uint64) string {
	if totalBytes > 0 && float64(freeBytes)/float64(totalBytes) < 0.10 {
		return lowDiskIcon
	}
	return diskIcon
}

func updateFreeSpace() {
	freeBytes, totalBytes, err := diskSpace()
	if err != nil {
		fmt.Printf("error reading disk space: %v\n", err)
		return
	}

	freeGB := formatBytesToGb(int64(freeBytes))
	freePercent := 0.0
	if totalBytes > 0 {
		freePercent = 100 * float64(freeBytes) / float64(totalBytes)
	}

	systray.SetTitle(fmt.Sprintf("%s %s", iconForFreeSpace(freeBytes, totalBytes), freeGB))
	tooltip := fmt.Sprintf("Free disk space is %s (%.0f%% free, %d Bytes)", freeGB, freePercent, freeBytes)
	systray.SetTooltip(tooltip)

}

func onReady() {

	systray.SetTitleFont(12, true)
	systray.SetTitle(fmt.Sprintf("%s init", diskIcon))
	mQuit := systray.AddMenuItem("Quit", "Quit the whole app")

	go func() {
		<-mQuit.ClickedCh
		systray.Quit()
	}()

	go func() {
		ticker := time.NewTicker(10 * time.Second)
		defer ticker.Stop()

		for range ticker.C {
			updateFreeSpace()
		}
	}()

	updateFreeSpace()
}

func onExit() {
	// clean up here
}
