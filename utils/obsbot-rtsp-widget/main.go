package main

import (
	"context"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"time"

	"github.com/getlantern/systray"
)

const (
	runningIcon = "􀌬"
	stoppedIcon = "􀙦"
	warningIcon = "􀇿"
)

type appConfig struct {
	ServiceLabel string
	PlistPath    string
	LogPath      string
	RTSPURL      string
	ScryptedURL  string
}

type serviceStatus struct {
	Loaded  bool
	Running bool
	State   string
	Detail  string
}

var (
	cfg         appConfig
	statusItem  *systray.MenuItem
	startItem   *systray.MenuItem
	stopItem    *systray.MenuItem
	restartItem *systray.MenuItem
	probeItem   *systray.MenuItem
)

func main() {
	cfg = loadConfig()
	systray.Run(onReady, onExit)
}

func loadConfig() appConfig {
	home, err := os.UserHomeDir()
	if err != nil {
		home = "/Users/nom"
	}

	return appConfig{
		ServiceLabel: env("OBSBOT_RTSP_SERVICE_LABEL", "local.obsbot-rtsp"),
		PlistPath:    env("OBSBOT_RTSP_SERVICE_PLIST", filepath.Join(home, "Library/LaunchAgents/local.obsbot-rtsp.plist")),
		LogPath:      env("OBSBOT_RTSP_LOG", filepath.Join(home, "Library/Logs/obsbot-rtsp.log")),
		RTSPURL:      env("OBSBOT_RTSP_URL", "rtsp://127.0.0.1:8554/obsbot"),
		ScryptedURL:  env("SCRYPTED_URL", "https://localhost:10443/"),
	}
}

func env(key, fallback string) string {
	value := os.Getenv(key)
	if value == "" {
		return fallback
	}
	return value
}

func guiDomain() string {
	return fmt.Sprintf("gui/%d", os.Getuid())
}

func serviceTarget() string {
	return fmt.Sprintf("%s/%s", guiDomain(), cfg.ServiceLabel)
}

func runLaunchctl(args ...string) error {
	cmd := exec.Command("launchctl", args...)
	output, err := cmd.CombinedOutput()
	if err != nil {
		return fmt.Errorf("launchctl %s: %w: %s", strings.Join(args, " "), err, strings.TrimSpace(string(output)))
	}
	return nil
}

func readServiceStatus() serviceStatus {
	cmd := exec.Command("launchctl", "print", serviceTarget())
	output, err := cmd.CombinedOutput()
	if err != nil {
		return serviceStatus{
			Loaded: false,
			State:  "unloaded",
			Detail: strings.TrimSpace(string(output)),
		}
	}

	state := parseLaunchState(string(output))
	if state == "" {
		state = "unknown"
	}

	return serviceStatus{
		Loaded:  true,
		Running: state == "running",
		State:   state,
		Detail:  fmt.Sprintf("%s is %s", cfg.ServiceLabel, state),
	}
}

func parseLaunchState(output string) string {
	for _, line := range strings.Split(output, "\n") {
		line = strings.TrimSpace(line)
		if strings.HasPrefix(line, "state = ") {
			return strings.TrimSpace(strings.TrimPrefix(line, "state = "))
		}
	}
	return ""
}

func titleForStatus(status serviceStatus) string {
	switch {
	case status.Running:
		return fmt.Sprintf("%s RTSP", runningIcon)
	case status.Loaded:
		return fmt.Sprintf("%s RTSP", warningIcon)
	default:
		return fmt.Sprintf("%s RTSP", stoppedIcon)
	}
}

func tooltipForStatus(status serviceStatus) string {
	switch {
	case status.Running:
		return fmt.Sprintf("OBSBot RTSP is running: %s", cfg.RTSPURL)
	case status.Loaded:
		return fmt.Sprintf("OBSBot RTSP is loaded but %s", status.State)
	default:
		return "OBSBot RTSP is not loaded"
	}
}

func updateStatus(message string) serviceStatus {
	status := readServiceStatus()
	title := titleForStatus(status)
	tooltip := tooltipForStatus(status)
	if message != "" {
		tooltip = message
	}

	systray.SetTitle(title)
	systray.SetTooltip(tooltip)
	statusItem.SetTitle(fmt.Sprintf("Status: %s", status.State))
	statusItem.SetTooltip(tooltip)

	if status.Running {
		startItem.Disable()
		stopItem.Enable()
		restartItem.Enable()
	} else {
		startItem.Enable()
		if status.Loaded {
			stopItem.Enable()
			restartItem.Enable()
		} else {
			stopItem.Disable()
			restartItem.Enable()
		}
	}

	probeItem.Enable()
	return status
}

func startService() error {
	if _, err := os.Stat(cfg.PlistPath); err != nil {
		return fmt.Errorf("missing LaunchAgent plist: %s", cfg.PlistPath)
	}

	if err := runLaunchctl("bootstrap", guiDomain(), cfg.PlistPath); err != nil {
		if !readServiceStatus().Loaded {
			return err
		}
	}
	if err := runLaunchctl("enable", serviceTarget()); err != nil {
		return err
	}
	return runLaunchctl("kickstart", "-k", serviceTarget())
}

func stopService() error {
	err := runLaunchctl("bootout", serviceTarget())
	if err == nil || !readServiceStatus().Loaded {
		return nil
	}

	if plistErr := runLaunchctl("bootout", guiDomain(), cfg.PlistPath); plistErr != nil && readServiceStatus().Loaded {
		return err
	}
	return nil
}

func restartService() error {
	if err := stopService(); err != nil {
		return err
	}
	time.Sleep(500 * time.Millisecond)
	return startService()
}

func copyRTSPURL() error {
	cmd := exec.Command("pbcopy")
	cmd.Stdin = strings.NewReader(cfg.RTSPURL)
	if err := cmd.Run(); err != nil {
		return err
	}
	return nil
}

func openPath(path string) error {
	cmd := exec.Command("open", path)
	return cmd.Start()
}

func openLog() error {
	if err := os.MkdirAll(filepath.Dir(cfg.LogPath), 0755); err != nil {
		return err
	}
	file, err := os.OpenFile(cfg.LogPath, os.O_CREATE, 0644)
	if err != nil {
		return err
	}
	_ = file.Close()
	return openPath(cfg.LogPath)
}

func probeRTSP() error {
	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	cmd := exec.CommandContext(ctx, "ffprobe",
		"-hide_banner",
		"-v", "error",
		"-rtsp_transport", "tcp",
		"-select_streams", "v:0",
		"-show_entries", "stream=codec_name,width,height",
		"-of", "compact=p=0:nk=1",
		cfg.RTSPURL,
	)
	output, err := cmd.CombinedOutput()
	if ctx.Err() == context.DeadlineExceeded {
		return fmt.Errorf("RTSP probe timed out")
	}
	if err != nil {
		return fmt.Errorf("RTSP probe failed: %s", strings.TrimSpace(string(output)))
	}

	result := strings.TrimSpace(string(output))
	if result == "" {
		result = "stream found"
	}
	updateStatus(fmt.Sprintf("RTSP OK: %s", result))
	return nil
}

func handleMenu(item *systray.MenuItem, action func() error, success string) {
	go func() {
		for range item.ClickedCh {
			item.Disable()
			if err := action(); err != nil {
				fmt.Printf("obsbot-rtsp-widget: %v\n", err)
				updateStatus(err.Error())
			} else {
				updateStatus(success)
			}
			if item != startItem && item != stopItem && item != restartItem && item != probeItem {
				item.Enable()
			}
		}
	}()
}

func onReady() {
	systray.SetTitle(fmt.Sprintf("%s RTSP", warningIcon))
	systray.SetTooltip("Checking OBSBot RTSP")

	statusItem = systray.AddMenuItem("Status: checking", "Current launchd status")
	statusItem.Disable()
	systray.AddSeparator()

	startItem = systray.AddMenuItem("Start OBSBot RTSP", "Start the local.obsbot-rtsp LaunchAgent")
	stopItem = systray.AddMenuItem("Stop OBSBot RTSP", "Stop the local.obsbot-rtsp LaunchAgent")
	restartItem = systray.AddMenuItem("Restart OBSBot RTSP", "Restart the local.obsbot-rtsp LaunchAgent")
	probeItem = systray.AddMenuItem("Probe RTSP Now", "Run a one-shot ffprobe against the local stream")
	systray.AddSeparator()

	copyItem := systray.AddMenuItem("Copy RTSP URL", cfg.RTSPURL)
	openScryptedItem := systray.AddMenuItem("Open Scrypted", cfg.ScryptedURL)
	openLogItem := systray.AddMenuItem("Open Publisher Log", cfg.LogPath)
	systray.AddSeparator()

	quitItem := systray.AddMenuItem("Quit", "Quit the OBSBot RTSP widget")

	handleMenu(startItem, startService, "Started OBSBot RTSP")
	handleMenu(stopItem, stopService, "Stopped OBSBot RTSP")
	handleMenu(restartItem, restartService, "Restarted OBSBot RTSP")
	handleMenu(probeItem, probeRTSP, "RTSP probe completed")
	handleMenu(copyItem, copyRTSPURL, "Copied RTSP URL")
	handleMenu(openScryptedItem, func() error { return openPath(cfg.ScryptedURL) }, "Opened Scrypted")
	handleMenu(openLogItem, openLog, "Opened publisher log")

	go func() {
		<-quitItem.ClickedCh
		systray.Quit()
	}()

	go func() {
		ticker := time.NewTicker(10 * time.Second)
		defer ticker.Stop()
		for range ticker.C {
			updateStatus("")
		}
	}()

	updateStatus("")
}

func onExit() {}
