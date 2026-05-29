package main

import (
	"fmt"
	"math"
	"os/exec"
	"time"

	"github.com/getlantern/systray"
	"golang.org/x/sys/unix"
)

const (
	uptimeIcon     = "􀣔"
	longUptimeIcon = "􀱨"
)

func main() {
	systray.Run(onReady, onExit)
}

func uptime() (time.Duration, error) {
	bootTime, err := unix.SysctlTimeval("kern.boottime")
	if err != nil {
		return 0, err
	}

	bootedAt := time.Unix(bootTime.Sec, int64(bootTime.Usec)*1000)
	return time.Since(bootedAt), nil
}

func formatUptime(duration time.Duration) string {
	minutes := int(math.Round(duration.Minutes()))
	if minutes < 1 {
		minutes = 1
	}
	if minutes < 60 {
		return fmt.Sprintf("%dm", minutes)
	}

	hours := int(math.Round(duration.Hours()))
	if hours < 24 {
		return fmt.Sprintf("%dh", hours)
	}

	days := int(math.Round(duration.Hours() / 24))
	return fmt.Sprintf("%dd", days)
}

func iconForUptime(duration time.Duration) string {
	if duration > 48*time.Hour {
		return longUptimeIcon
	}
	return uptimeIcon
}

func updateUptime() {
	duration, err := uptime()
	if err != nil {
		fmt.Printf("error reading uptime: %v\n", err)
		return
	}

	formatted := formatUptime(duration)
	systray.SetTitle(fmt.Sprintf("%s %s", iconForUptime(duration), formatted))
	systray.SetTooltip(fmt.Sprintf("System uptime is %s", formatted))
}

func runPowerAction(label, command string) {
	script := fmt.Sprintf(`display dialog "Are you sure you want to %s this Mac?" buttons {"Cancel", "%s"} default button "Cancel" cancel button "Cancel" with icon caution
tell application "System Events" to %s`, label, label, command)

	if err := exec.Command("osascript", "-e", script).Run(); err != nil {
		fmt.Printf("error running %s action: %v\n", label, err)
	}
}

func onReady() {
	systray.SetTitle(fmt.Sprintf("%s init", uptimeIcon))
	mReboot := systray.AddMenuItem("Reboot", "Reboot this Mac")
	mShutdown := systray.AddMenuItem("Shutdown", "Shut down this Mac")
	systray.AddSeparator()
	mQuit := systray.AddMenuItem("Quit", "Quit the uptime widget")

	go func() {
		<-mQuit.ClickedCh
		systray.Quit()
	}()

	go func() {
		for range mReboot.ClickedCh {
			go runPowerAction("Reboot", "restart")
		}
	}()

	go func() {
		for range mShutdown.ClickedCh {
			go runPowerAction("Shutdown", "shut down")
		}
	}()

	go func() {
		ticker := time.NewTicker(30 * time.Second)
		defer ticker.Stop()

		for range ticker.C {
			updateUptime()
		}
	}()

	updateUptime()
}

func onExit() {
	// clean up here
}
