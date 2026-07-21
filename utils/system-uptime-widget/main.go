package main

import (
	"fmt"
	"os/exec"
	"time"

	"github.com/getlantern/systray"
	"golang.org/x/sys/unix"
)

const uptimeIcon = "􂝔"

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

// formatUptime renders a duration as compact "<days>d <hours>h", omitting a
// unit when it is zero: e.g. "2d 1h", "33d 3h", "1d", "1h".
func formatUptime(duration time.Duration) string {
	totalHours := int(duration.Hours())
	days := totalHours / 24
	hours := totalHours % 24

	switch {
	case days > 0 && hours > 0:
		return fmt.Sprintf("%dd %dh", days, hours)
	case days > 0:
		return fmt.Sprintf("%dd", days)
	default:
		return fmt.Sprintf("%dh", hours)
	}
}

// humanUptime renders a duration as a readable phrase for the tooltip, e.g.
// "2 days, 1 hour".
func humanUptime(duration time.Duration) string {
	totalHours := int(duration.Hours())
	days := totalHours / 24
	hours := totalHours % 24

	plural := func(n int, unit string) string {
		if n == 1 {
			return fmt.Sprintf("%d %s", n, unit)
		}
		return fmt.Sprintf("%d %ss", n, unit)
	}

	switch {
	case days > 0 && hours > 0:
		return fmt.Sprintf("%s, %s", plural(days, "day"), plural(hours, "hour"))
	case days > 0:
		return plural(days, "day")
	default:
		return plural(hours, "hour")
	}
}

func updateUptime() {
	duration, err := uptime()
	if err != nil {
		fmt.Printf("error reading uptime: %v\n", err)
		return
	}

	systray.SetTitle(fmt.Sprintf("%s %s", uptimeIcon, formatUptime(duration)))
	systray.SetTooltip(fmt.Sprintf("System uptime is %s", humanUptime(duration)))
}

func runPowerAction(label, command string) {
	script := fmt.Sprintf(`display dialog "Are you sure you want to %s this Mac?" buttons {"Cancel", "%s"} default button "Cancel" cancel button "Cancel" with icon caution
tell application "System Events" to %s`, label, label, command)

	if err := exec.Command("osascript", "-e", script).Run(); err != nil {
		fmt.Printf("error running %s action: %v\n", label, err)
	}
}

func onReady() {
	// Match free-disk-space-widget: render at the Control Center text size.
	systray.SetTitleFont(11, false)
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
