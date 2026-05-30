package main

import (
	"fmt"
	"os/exec"
	"time"

	"github.com/getlantern/systray"
)

const (
	diskIcon    = "􀤂"
	lowDiskIcon = "􁘥"
)

func main() {
	systray.Run(onReady, onExit)
}

func formatBytesToGb(bytes int64) string {
	return fmt.Sprintf("%.2f GB", float64(bytes)/1_000_000_000)
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

func openApplication(appName string) {
	cmd := exec.Command("open", "-a", appName)
	if err := cmd.Start(); err != nil {
		fmt.Printf("error opening %s: %v\n", appName, err)
	}
}

func onReady() {

	systray.SetTitleFont(12, true)
	systray.SetTitle(fmt.Sprintf("%s init", diskIcon))
	mDiskUtility := systray.AddMenuItem("Open Disk Utility", "Open Disk Utility")
	mDaisyDisk := systray.AddMenuItem("Open DaisyDisk", "Open DaisyDisk")
	systray.AddSeparator()
	mQuit := systray.AddMenuItem("Quit", "Quit the whole app")

	go func() {
		for range mDiskUtility.ClickedCh {
			openApplication("Disk Utility")
		}
	}()

	go func() {
		for range mDaisyDisk.ClickedCh {
			openApplication("DaisyDisk")
		}
	}()

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
