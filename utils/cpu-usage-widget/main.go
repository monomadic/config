package main

import (
	"bytes"
	"encoding/json"
	"fmt"
	"image"
	"image/color"
	"image/png"
	"math"
	"os"
	"os/exec"
	"path/filepath"
	"sync"
	"time"

	"github.com/getlantern/systray"
)

const (
	configDirName  = "cpu-usage-widget"
	configFileName = "settings.json"
	iconScale      = 3

	pollInterval = 2 * time.Second
)

type layoutStyle string

const (
	stylePerCore layoutStyle = "per_core"
	styleBar     layoutStyle = "bar"
	styleBarText layoutStyle = "bar_text"
	styleText    layoutStyle = "text"
)

type widgetSettings struct {
	Style layoutStyle `json:"style"`
}

var (
	settingsMu sync.RWMutex
	settings   = defaultSettings()

	styleMenuItems = map[layoutStyle]*systray.MenuItem{}
)

func main() {
	systray.Run(onReady, onExit)
}

func defaultSettings() widgetSettings {
	return widgetSettings{Style: stylePerCore}
}

func allLayoutStyles() []layoutStyle {
	return []layoutStyle{stylePerCore, styleBar, styleBarText, styleText}
}

func (style layoutStyle) label() string {
	switch style {
	case stylePerCore:
		return "Per-core Bars"
	case styleBar:
		return "Aggregate Bar"
	case styleBarText:
		return "Aggregate Bar and Text"
	case styleText:
		return "Percentage Text"
	default:
		return string(style)
	}
}

func (style layoutStyle) valid() bool {
	for _, option := range allLayoutStyles() {
		if style == option {
			return true
		}
	}
	return false
}

func configPath() (string, error) {
	userConfigDir, err := os.UserConfigDir()
	if err != nil {
		return "", err
	}
	return filepath.Join(userConfigDir, configDirName, configFileName), nil
}

func loadSettings() widgetSettings {
	loadedSettings := defaultSettings()

	path, err := configPath()
	if err != nil {
		fmt.Printf("error finding config directory: %v\n", err)
		return loadedSettings
	}

	configFile, err := os.Open(path)
	if err != nil {
		if !os.IsNotExist(err) {
			fmt.Printf("error opening settings: %v\n", err)
		}
		return loadedSettings
	}
	defer configFile.Close()

	if err := json.NewDecoder(configFile).Decode(&loadedSettings); err != nil {
		fmt.Printf("error reading settings: %v\n", err)
		return defaultSettings()
	}

	if !loadedSettings.Style.valid() {
		loadedSettings.Style = defaultSettings().Style
	}

	return loadedSettings
}

func saveSettings(settingsToSave widgetSettings) {
	path, err := configPath()
	if err != nil {
		fmt.Printf("error finding config directory: %v\n", err)
		return
	}

	if err := os.MkdirAll(filepath.Dir(path), 0o755); err != nil {
		fmt.Printf("error creating config directory: %v\n", err)
		return
	}

	data, err := json.MarshalIndent(settingsToSave, "", "  ")
	if err != nil {
		fmt.Printf("error encoding settings: %v\n", err)
		return
	}

	if err := os.WriteFile(path, data, 0o644); err != nil {
		fmt.Printf("error saving settings: %v\n", err)
	}
}

func getSettings() widgetSettings {
	settingsMu.RLock()
	defer settingsMu.RUnlock()
	return settings
}

func setSettings(nextSettings widgetSettings) {
	settingsMu.Lock()
	settings = nextSettings
	settingsMu.Unlock()

	saveSettings(nextSettings)
	refreshMenuChecks()
	updateCPU()
}

func setLayoutStyle(style layoutStyle) {
	if !style.valid() {
		return
	}

	nextSettings := getSettings()
	nextSettings.Style = style
	setSettings(nextSettings)
}

func aggregateUsage(usage []float64) float64 {
	if len(usage) == 0 {
		return 0
	}
	var sum float64
	for _, u := range usage {
		sum += clampRatio(u)
	}
	return sum / float64(len(usage))
}

func clampRatio(ratio float64) float64 {
	return math.Max(0, math.Min(1, ratio))
}

func formatPercent(ratio float64) string {
	return fmt.Sprintf("%.0f%%", clampRatio(ratio)*100)
}

func menuBarTitle(usage []float64, style layoutStyle) string {
	switch style {
	case styleText, styleBarText:
		return formatPercent(aggregateUsage(usage))
	default:
		return ""
	}
}

func updateStatusIcon(usage []float64, style layoutStyle) {
	switch style {
	case stylePerCore:
		setTemplateStatusIcon(renderCoreBars(usage))
	case styleBar, styleBarText:
		setTemplateStatusIcon(renderAggregateBar(usage))
	default:
		systray.ClearIcon()
	}
}

func setTemplateStatusIcon(iconBytes []byte, widthPoints, heightPoints int) {
	if len(iconBytes) == 0 {
		return
	}
	systray.SetTemplateIconWithSize(iconBytes, iconBytes, float64(widthPoints), float64(heightPoints))
}

// renderCoreBars draws one vertical bar per core: a faint full-height track with
// a solid fill rising from the bottom in proportion to that core's load.
func renderCoreBars(usage []float64) ([]byte, int, int) {
	cores := usage
	if len(cores) == 0 {
		cores = []float64{0}
	}

	const (
		barPoints = 2
		gapPoints = 2
		heightPts = 14
		padPoints = 1
	)

	scale := iconScale
	barW := barPoints * scale
	gap := gapPoints * scale
	widthPx := len(cores)*barW + (len(cores)-1)*gap
	heightPx := heightPts * scale

	img := image.NewNRGBA(image.Rect(0, 0, widthPx, heightPx))

	inactive := color.NRGBA{A: 70}
	active := color.NRGBA{A: 255}

	trackTop := padPoints * scale
	trackBottom := heightPx - padPoints*scale
	trackHeight := trackBottom - trackTop

	for i, ratio := range cores {
		x := i * (barW + gap)
		drawBarColumn(img, x, trackTop, barW, trackHeight, inactive)

		fill := int(math.Round(clampRatio(ratio) * float64(trackHeight)))
		drawBarColumn(img, x, trackBottom-fill, barW, fill, active)
	}

	widthPoints := len(cores)*barPoints + (len(cores)-1)*gapPoints
	return pngBytes(img), widthPoints, heightPts
}

func renderAggregateBar(usage []float64) ([]byte, int, int) {
	const (
		widthPoints  = 34
		heightPoints = 12
	)

	scale := iconScale
	img := image.NewNRGBA(image.Rect(0, 0, widthPoints*scale, heightPoints*scale))

	barHeight := 5 * scale
	barWidth := (widthPoints - 2) * scale
	x := scale
	y := (heightPoints*scale - barHeight) / 2

	drawProgressBar(img, x, y, barWidth, barHeight, aggregateUsage(usage))
	return pngBytes(img), widthPoints, heightPoints
}

func pngBytes(img image.Image) []byte {
	var buf bytes.Buffer
	if err := png.Encode(&buf, img); err != nil {
		fmt.Printf("error rendering status icon: %v\n", err)
		return nil
	}
	return buf.Bytes()
}

func drawProgressBar(img *image.NRGBA, x, y, width, height int, ratio float64) {
	inactive := color.NRGBA{A: 78}
	active := color.NRGBA{A: 255}
	radius := height / 2
	fillLimit := x + int(math.Round(clampRatio(ratio)*float64(width)))

	for yy := y; yy < y+height; yy++ {
		for xx := x; xx < x+width; xx++ {
			if !inRoundedRect(xx, yy, x, y, x+width, y+height, radius) {
				continue
			}
			if xx < fillLimit {
				img.SetNRGBA(xx, yy, active)
			} else {
				img.SetNRGBA(xx, yy, inactive)
			}
		}
	}
}

func drawBarColumn(img *image.NRGBA, x, y, width, height int, c color.NRGBA) {
	if height <= 0 {
		return
	}
	radius := width / 2
	if height/2 < radius {
		radius = height / 2
	}
	drawRoundedRect(img, x, y, width, height, radius, c)
}

func drawRoundedRect(img *image.NRGBA, x, y, width, height, radius int, c color.NRGBA) {
	x1 := x + width
	y1 := y + height

	for yy := y; yy < y1; yy++ {
		for xx := x; xx < x1; xx++ {
			if inRoundedRect(xx, yy, x, y, x1, y1, radius) {
				img.SetNRGBA(xx, yy, c)
			}
		}
	}
}

func inRoundedRect(x, y, x0, y0, x1, y1, radius int) bool {
	if x < x0 || x >= x1 || y < y0 || y >= y1 {
		return false
	}
	if radius <= 0 {
		return true
	}
	if x >= x0+radius && x < x1-radius {
		return true
	}
	if y >= y0+radius && y < y1-radius {
		return true
	}

	cx := x0 + radius
	if x >= x1-radius {
		cx = x1 - radius - 1
	}

	cy := y0 + radius
	if y >= y1-radius {
		cy = y1 - radius - 1
	}

	dx := x - cx
	dy := y - cy
	return dx*dx+dy*dy <= radius*radius
}

func updateCPU() {
	usage, err := perCoreUsage()
	if err != nil {
		fmt.Printf("error reading cpu usage: %v\n", err)
		return
	}

	currentSettings := getSettings()

	systray.SetTitleFont(11, false)
	updateStatusIcon(usage, currentSettings.Style)
	systray.SetTitle(menuBarTitle(usage, currentSettings.Style))

	tooltip := fmt.Sprintf("CPU %s across %d cores", formatPercent(aggregateUsage(usage)), len(usage))
	systray.SetTooltip(tooltip)
}

func openApplication(appName string) {
	cmd := exec.Command("open", "-a", appName)
	if err := cmd.Start(); err != nil {
		fmt.Printf("error opening %s: %v\n", appName, err)
	}
}

func onReady() {
	settingsMu.Lock()
	settings = loadSettings()
	settingsMu.Unlock()

	systray.SetTitleFont(11, false)
	systray.SetTitle("CPU")

	// Prime the delta baseline so the first tick shows real utilization.
	if _, err := perCoreUsage(); err != nil {
		fmt.Printf("error priming cpu baseline: %v\n", err)
	}

	mStyle := systray.AddMenuItem("Style", "Choose the menu bar layout")
	for _, style := range allLayoutStyles() {
		styleMenuItems[style] = mStyle.AddSubMenuItem(style.label(), fmt.Sprintf("Use the %s layout", style.label()))
	}

	refreshMenuChecks()
	systray.AddSeparator()

	mActivityMonitor := systray.AddMenuItem("Open Activity Monitor", "Open Activity Monitor")
	systray.AddSeparator()
	mQuit := systray.AddMenuItem("Quit", "Quit the whole app")

	for style, menuItem := range styleMenuItems {
		style := style
		menuItem := menuItem
		go func() {
			for range menuItem.ClickedCh {
				setLayoutStyle(style)
			}
		}()
	}

	go func() {
		for range mActivityMonitor.ClickedCh {
			openApplication("Activity Monitor")
		}
	}()

	go func() {
		<-mQuit.ClickedCh
		systray.Quit()
	}()

	go func() {
		ticker := time.NewTicker(pollInterval)
		defer ticker.Stop()

		for range ticker.C {
			updateCPU()
		}
	}()

	updateCPU()
}

func onExit() {
	// clean up here
}

func refreshMenuChecks() {
	currentSettings := getSettings()

	for style, menuItem := range styleMenuItems {
		if style == currentSettings.Style {
			menuItem.Check()
		} else {
			menuItem.Uncheck()
		}
	}
}
