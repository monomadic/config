package main

import (
	"context"
	"crypto/tls"
	"fmt"
	"net/http"
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

type serviceConfig struct {
	Name      string
	Label     string
	PlistPath string
}

type appConfig struct {
	RTSP        serviceConfig
	Scrypted    serviceConfig
	LogPath     string
	RTSPURL     string
	ScryptedURL string
}

type serviceStatus struct {
	Loaded  bool
	Running bool
	State   string
	Detail  string
}

type serviceMenu struct {
	status  *systray.MenuItem
	start   *systray.MenuItem
	stop    *systray.MenuItem
	restart *systray.MenuItem
}

var (
	cfg               appConfig
	rtspMenu          serviceMenu
	scryptedMenu      serviceMenu
	probeRTSPItem     *systray.MenuItem
	probeScryptedItem *systray.MenuItem
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
		RTSP: serviceConfig{
			Name:      "OBSBot RTSP",
			Label:     env("OBSBOT_RTSP_SERVICE_LABEL", "local.obsbot-rtsp"),
			PlistPath: env("OBSBOT_RTSP_SERVICE_PLIST", filepath.Join(home, "Library/LaunchAgents/local.obsbot-rtsp.plist")),
		},
		Scrypted: serviceConfig{
			Name:      "Scrypted",
			Label:     env("SCRYPTED_SERVICE_LABEL", "app.scrypted.server"),
			PlistPath: env("SCRYPTED_SERVICE_PLIST", filepath.Join(home, "Library/LaunchAgents/app.scrypted.server.plist")),
		},
		LogPath:     env("OBSBOT_RTSP_LOG", filepath.Join(home, "Library/Logs/obsbot-rtsp.log")),
		RTSPURL:     env("OBSBOT_RTSP_URL", "rtsp://127.0.0.1:8554/obsbot"),
		ScryptedURL: env("SCRYPTED_URL", "https://localhost:10443/"),
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

func serviceTarget(service serviceConfig) string {
	return fmt.Sprintf("%s/%s", guiDomain(), service.Label)
}

func runLaunchctl(args ...string) error {
	cmd := exec.Command("launchctl", args...)
	output, err := cmd.CombinedOutput()
	if err != nil {
		return fmt.Errorf("launchctl %s: %w: %s", strings.Join(args, " "), err, strings.TrimSpace(string(output)))
	}
	return nil
}

func readServiceStatus(service serviceConfig) serviceStatus {
	cmd := exec.Command("launchctl", "print", serviceTarget(service))
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
		Detail:  fmt.Sprintf("%s is %s", service.Name, state),
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

func titleForStatuses(rtsp, scrypted serviceStatus) string {
	switch {
	case rtsp.Running && scrypted.Running:
		return fmt.Sprintf("%s RTSP", runningIcon)
	case rtsp.Running || scrypted.Running || rtsp.Loaded || scrypted.Loaded:
		return fmt.Sprintf("%s RTSP", warningIcon)
	default:
		return fmt.Sprintf("%s RTSP", stoppedIcon)
	}
}

func tooltipForStatuses(rtsp, scrypted serviceStatus) string {
	return fmt.Sprintf("OBSBot RTSP: %s; Scrypted: %s", rtsp.State, scrypted.State)
}

func updateServiceMenu(menu serviceMenu, service serviceConfig, status serviceStatus) {
	menu.status.SetTitle(fmt.Sprintf("%s: %s", service.Name, status.State))
	menu.status.SetTooltip(status.Detail)

	if status.Running {
		menu.start.Disable()
		menu.stop.Enable()
		menu.restart.Enable()
		return
	}

	menu.start.Enable()
	if status.Loaded {
		menu.stop.Enable()
		menu.restart.Enable()
	} else {
		menu.stop.Disable()
		menu.restart.Enable()
	}
}

func updateStatus(message string) (serviceStatus, serviceStatus) {
	rtsp := readServiceStatus(cfg.RTSP)
	scrypted := readServiceStatus(cfg.Scrypted)

	title := titleForStatuses(rtsp, scrypted)
	tooltip := tooltipForStatuses(rtsp, scrypted)
	if message != "" {
		tooltip = message
	}

	systray.SetTitle(title)
	systray.SetTooltip(tooltip)
	updateServiceMenu(rtspMenu, cfg.RTSP, rtsp)
	updateServiceMenu(scryptedMenu, cfg.Scrypted, scrypted)
	probeRTSPItem.Enable()
	probeScryptedItem.Enable()

	return rtsp, scrypted
}

func startService(service serviceConfig) error {
	if _, err := os.Stat(service.PlistPath); err != nil {
		return fmt.Errorf("missing LaunchAgent plist: %s", service.PlistPath)
	}

	if err := runLaunchctl("bootstrap", guiDomain(), service.PlistPath); err != nil {
		if !readServiceStatus(service).Loaded {
			return err
		}
	}
	if err := runLaunchctl("enable", serviceTarget(service)); err != nil {
		return err
	}
	return runLaunchctl("kickstart", "-k", serviceTarget(service))
}

func stopService(service serviceConfig) error {
	err := runLaunchctl("bootout", serviceTarget(service))
	if err == nil || !readServiceStatus(service).Loaded {
		return nil
	}

	if plistErr := runLaunchctl("bootout", guiDomain(), service.PlistPath); plistErr != nil && readServiceStatus(service).Loaded {
		return err
	}
	return nil
}

func restartService(service serviceConfig) error {
	if err := stopService(service); err != nil {
		return err
	}
	time.Sleep(500 * time.Millisecond)
	return startService(service)
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

func probeScrypted() error {
	transport := &http.Transport{
		TLSClientConfig: &tls.Config{InsecureSkipVerify: true},
	}
	client := http.Client{
		Timeout:   5 * time.Second,
		Transport: transport,
	}

	request, err := http.NewRequest(http.MethodHead, cfg.ScryptedURL, nil)
	if err != nil {
		return err
	}

	response, err := client.Do(request)
	if err != nil {
		return fmt.Errorf("Scrypted probe failed: %w", err)
	}
	defer response.Body.Close()

	if response.StatusCode < 200 || response.StatusCode >= 400 {
		return fmt.Errorf("Scrypted probe returned %s", response.Status)
	}

	updateStatus(fmt.Sprintf("Scrypted OK: %s", response.Status))
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
			if !isManagedServiceItem(item) {
				item.Enable()
			}
		}
	}()
}

func isManagedServiceItem(item *systray.MenuItem) bool {
	return item == rtspMenu.start ||
		item == rtspMenu.stop ||
		item == rtspMenu.restart ||
		item == scryptedMenu.start ||
		item == scryptedMenu.stop ||
		item == scryptedMenu.restart ||
		item == probeRTSPItem ||
		item == probeScryptedItem
}

func addServiceMenu(service serviceConfig) serviceMenu {
	menu := serviceMenu{
		status:  systray.AddMenuItem(fmt.Sprintf("%s: checking", service.Name), fmt.Sprintf("Current %s launchd status", service.Name)),
		start:   systray.AddMenuItem(fmt.Sprintf("Start %s", service.Name), fmt.Sprintf("Start %s", service.Label)),
		stop:    systray.AddMenuItem(fmt.Sprintf("Stop %s", service.Name), fmt.Sprintf("Stop %s", service.Label)),
		restart: systray.AddMenuItem(fmt.Sprintf("Restart %s", service.Name), fmt.Sprintf("Restart %s", service.Label)),
	}
	menu.status.Disable()
	return menu
}

func onReady() {
	systray.SetTitle(fmt.Sprintf("%s RTSP", warningIcon))
	systray.SetTooltip("Checking OBSBot RTSP and Scrypted")

	rtspMenu = addServiceMenu(cfg.RTSP)
	probeRTSPItem = systray.AddMenuItem("Probe RTSP Now", "Run a one-shot ffprobe against the local stream")
	systray.AddSeparator()

	scryptedMenu = addServiceMenu(cfg.Scrypted)
	probeScryptedItem = systray.AddMenuItem("Probe Scrypted Now", "Run a one-shot HTTPS probe against Scrypted")
	systray.AddSeparator()

	copyItem := systray.AddMenuItem("Copy RTSP URL", cfg.RTSPURL)
	openScryptedItem := systray.AddMenuItem("Open Scrypted", cfg.ScryptedURL)
	openLogItem := systray.AddMenuItem("Open Publisher Log", cfg.LogPath)
	systray.AddSeparator()

	quitItem := systray.AddMenuItem("Quit", "Quit the OBSBot RTSP widget")

	handleMenu(rtspMenu.start, func() error { return startService(cfg.RTSP) }, "Started OBSBot RTSP")
	handleMenu(rtspMenu.stop, func() error { return stopService(cfg.RTSP) }, "Stopped OBSBot RTSP")
	handleMenu(rtspMenu.restart, func() error { return restartService(cfg.RTSP) }, "Restarted OBSBot RTSP")
	handleMenu(probeRTSPItem, probeRTSP, "RTSP probe completed")
	handleMenu(scryptedMenu.start, func() error { return startService(cfg.Scrypted) }, "Started Scrypted")
	handleMenu(scryptedMenu.stop, func() error { return stopService(cfg.Scrypted) }, "Stopped Scrypted")
	handleMenu(scryptedMenu.restart, func() error { return restartService(cfg.Scrypted) }, "Restarted Scrypted")
	handleMenu(probeScryptedItem, probeScrypted, "Scrypted probe completed")
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
