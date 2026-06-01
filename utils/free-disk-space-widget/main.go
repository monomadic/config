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
	diskIcon    = "􀤂"
	lowDiskIcon = "􁘥"

	configDirName  = "free-disk-space-widget"
	configFileName = "settings.json"
	iconScale      = 3
)

type displayMode string

const (
	displayGB      displayMode = "gb"
	displayPercent displayMode = "percent"
)

type layoutStyle string

const (
	styleText     layoutStyle = "text"
	styleIconText layoutStyle = "icon_text"
	styleBarText  layoutStyle = "bar_text"
	styleBarIcon  layoutStyle = "bar_icon"
	styleBar      layoutStyle = "bar"
)

type widgetSettings struct {
	Style   layoutStyle `json:"style"`
	Display displayMode `json:"display"`
}

var (
	settingsMu sync.RWMutex
	settings   = defaultSettings()

	styleMenuItems   = map[layoutStyle]*systray.MenuItem{}
	displayMenuItems = map[displayMode]*systray.MenuItem{}
)

func main() {
	systray.Run(onReady, onExit)
}

func defaultSettings() widgetSettings {
	return widgetSettings{
		Style:   styleIconText,
		Display: displayGB,
	}
}

func allLayoutStyles() []layoutStyle {
	return []layoutStyle{
		styleText,
		styleIconText,
		styleBarText,
		styleBarIcon,
		styleBar,
	}
}

func allDisplayModes() []displayMode {
	return []displayMode{
		displayGB,
		displayPercent,
	}
}

func (style layoutStyle) label() string {
	switch style {
	case styleText:
		return "Text"
	case styleIconText:
		return "Icon and Text"
	case styleBarText:
		return "Bar and Text"
	case styleBarIcon:
		return "Bar and Icon"
	case styleBar:
		return "Bar"
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

func (mode displayMode) label() string {
	switch mode {
	case displayGB:
		return "GB"
	case displayPercent:
		return "Percentage"
	default:
		return string(mode)
	}
}

func (mode displayMode) valid() bool {
	for _, option := range allDisplayModes() {
		if mode == option {
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
	if !loadedSettings.Display.valid() {
		loadedSettings.Display = defaultSettings().Display
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
	updateFreeSpace()
}

func setLayoutStyle(style layoutStyle) {
	if !style.valid() {
		return
	}

	nextSettings := getSettings()
	nextSettings.Style = style
	setSettings(nextSettings)
}

func setDisplayMode(mode displayMode) {
	if !mode.valid() {
		return
	}

	nextSettings := getSettings()
	nextSettings.Display = mode
	setSettings(nextSettings)
}

func formatBytesToGb(bytes int64) string {
	gb := math.Round(float64(bytes) / 1_000_000_000)
	return fmt.Sprintf("%.0f GB", gb)
}

func formatPercent(percent float64) string {
	return fmt.Sprintf("%.0f%%", percent)
}

func iconForFreeSpace(freeBytes, totalBytes uint64) string {
	if totalBytes > 0 && float64(freeBytes)/float64(totalBytes) < 0.10 {
		return lowDiskIcon
	}
	return diskIcon
}

func freePercentValue(freeBytes, totalBytes uint64) float64 {
	return 100 * freeRatioValue(freeBytes, totalBytes)
}

func freeRatioValue(freeBytes, totalBytes uint64) float64 {
	if totalBytes == 0 {
		return 0
	}

	ratio := float64(freeBytes) / float64(totalBytes)
	return math.Max(0, math.Min(1, ratio))
}

func formattedFreeSpaceValue(freeGB string, freePercent float64, mode displayMode) string {
	switch mode {
	case displayPercent:
		return formatPercent(freePercent)
	default:
		return freeGB
	}
}

func menuBarTitle(freeBytes, totalBytes uint64, settings widgetSettings) string {
	freeGB := formatBytesToGb(int64(freeBytes))
	freePercent := freePercentValue(freeBytes, totalBytes)
	value := formattedFreeSpaceValue(freeGB, freePercent, settings.Display)
	icon := iconForFreeSpace(freeBytes, totalBytes)

	switch settings.Style {
	case styleText:
		return value
	case styleIconText:
		return fmt.Sprintf("%s %s", icon, value)
	case styleBarText:
		return value
	default:
		return ""
	}
}

func updateTitleFont(style layoutStyle) {
	systray.SetTitleFont(12, true)
}

func updateStatusIcon(freeBytes, totalBytes uint64, style layoutStyle) {
	switch style {
	case styleBarText:
		setTemplateStatusIcon(renderBarIcon(freeBytes, totalBytes, 32, 12), 32, 12)
	case styleBarIcon:
		setTemplateStatusIcon(renderBarAndIcon(freeBytes, totalBytes), 46, 20)
	case styleBar:
		setTemplateStatusIcon(renderBarIcon(freeBytes, totalBytes, 42, 12), 42, 12)
	default:
		systray.ClearIcon()
	}
}

func setTemplateStatusIcon(iconBytes []byte, width float64, height float64) {
	if len(iconBytes) == 0 {
		return
	}

	systray.SetTemplateIconWithSize(iconBytes, iconBytes, width, height)
}

func renderBarIcon(freeBytes, totalBytes uint64, widthPoints int, heightPoints int) []byte {
	img := newTemplateImage(widthPoints, heightPoints)
	scale := iconScale
	barHeight := 5 * scale
	barWidth := (widthPoints - 2) * scale
	x := scale
	y := (heightPoints*scale - barHeight) / 2

	drawProgressBar(img, x, y, barWidth, barHeight, freeRatioValue(freeBytes, totalBytes))
	return pngBytes(img)
}

func renderBarAndIcon(freeBytes, totalBytes uint64) []byte {
	const (
		widthPoints  = 46
		heightPoints = 20
	)

	img := newTemplateImage(widthPoints, heightPoints)
	scale := iconScale

	iconWidth := 14 * scale
	iconHeight := 9 * scale
	iconX := (widthPoints*scale - iconWidth) / 2
	drawDriveIcon(img, iconX, scale, iconWidth, iconHeight)

	barWidth := 38 * scale
	barHeight := 4 * scale
	barX := (widthPoints*scale - barWidth) / 2
	barY := 14 * scale
	drawProgressBar(img, barX, barY, barWidth, barHeight, freeRatioValue(freeBytes, totalBytes))

	return pngBytes(img)
}

func newTemplateImage(widthPoints int, heightPoints int) *image.NRGBA {
	return image.NewNRGBA(image.Rect(0, 0, widthPoints*iconScale, heightPoints*iconScale))
}

func pngBytes(img image.Image) []byte {
	var buf bytes.Buffer
	if err := png.Encode(&buf, img); err != nil {
		fmt.Printf("error rendering status icon: %v\n", err)
		return nil
	}
	return buf.Bytes()
}

func drawProgressBar(img *image.NRGBA, x int, y int, width int, height int, ratio float64) {
	inactive := color.NRGBA{A: 78}
	active := color.NRGBA{A: 255}
	radius := height / 2
	fillLimit := x + int(math.Round(ratio*float64(width)))

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

func drawDriveIcon(img *image.NRGBA, x int, y int, width int, height int) {
	active := color.NRGBA{A: 255}
	scale := iconScale
	stroke := scale

	drawRoundedRectStroke(img, x, y, width, height, 3*scale, stroke, active)
	drawRect(img, x+4*scale, y+2*scale, width-8*scale, stroke, active)
	drawCircle(img, x+width-4*scale, y+height-3*scale, scale, active)
}

func drawRoundedRectStroke(img *image.NRGBA, x int, y int, width int, height int, radius int, stroke int, c color.NRGBA) {
	transparent := color.NRGBA{}

	drawRoundedRect(img, x, y, width, height, radius, c)
	drawRoundedRect(img, x+stroke, y+stroke, width-2*stroke, height-2*stroke, radius-stroke, transparent)
}

func drawRoundedRect(img *image.NRGBA, x int, y int, width int, height int, radius int, c color.NRGBA) {
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

func inRoundedRect(x int, y int, x0 int, y0 int, x1 int, y1 int, radius int) bool {
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

func drawRect(img *image.NRGBA, x int, y int, width int, height int, c color.NRGBA) {
	for yy := y; yy < y+height; yy++ {
		for xx := x; xx < x+width; xx++ {
			img.SetNRGBA(xx, yy, c)
		}
	}
}

func drawCircle(img *image.NRGBA, centerX int, centerY int, radius int, c color.NRGBA) {
	for yy := centerY - radius; yy <= centerY+radius; yy++ {
		for xx := centerX - radius; xx <= centerX+radius; xx++ {
			dx := xx - centerX
			dy := yy - centerY
			if dx*dx+dy*dy <= radius*radius {
				img.SetNRGBA(xx, yy, c)
			}
		}
	}
}

func updateFreeSpace() {
	freeBytes, totalBytes, err := diskSpace()
	if err != nil {
		fmt.Printf("error reading disk space: %v\n", err)
		return
	}

	freeGB := formatBytesToGb(int64(freeBytes))
	freePercent := freePercentValue(freeBytes, totalBytes)
	currentSettings := getSettings()

	updateTitleFont(currentSettings.Style)
	updateStatusIcon(freeBytes, totalBytes, currentSettings.Style)
	systray.SetTitle(menuBarTitle(freeBytes, totalBytes, currentSettings))
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

	settingsMu.Lock()
	settings = loadSettings()
	settingsMu.Unlock()

	updateTitleFont(getSettings().Style)
	systray.SetTitle(fmt.Sprintf("%s init", diskIcon))

	mStyle := systray.AddMenuItem("Style", "Choose the menu bar layout")
	for _, style := range allLayoutStyles() {
		styleMenuItems[style] = mStyle.AddSubMenuItem(style.label(), fmt.Sprintf("Use the %s layout", style.label()))
	}

	mDisplay := systray.AddMenuItem("Show Free Space As", "Choose how free space text is displayed")
	for _, mode := range allDisplayModes() {
		displayMenuItems[mode] = mDisplay.AddSubMenuItem(mode.label(), fmt.Sprintf("Show free space as %s", mode.label()))
	}

	refreshMenuChecks()
	systray.AddSeparator()

	mDiskUtility := systray.AddMenuItem("Open Disk Utility", "Open Disk Utility")
	mDaisyDisk := systray.AddMenuItem("Open DaisyDisk", "Open DaisyDisk")
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

	for mode, menuItem := range displayMenuItems {
		mode := mode
		menuItem := menuItem
		go func() {
			for range menuItem.ClickedCh {
				setDisplayMode(mode)
			}
		}()
	}

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

func refreshMenuChecks() {
	currentSettings := getSettings()

	for style, menuItem := range styleMenuItems {
		if style == currentSettings.Style {
			menuItem.Check()
		} else {
			menuItem.Uncheck()
		}
	}

	for mode, menuItem := range displayMenuItems {
		if mode == currentSettings.Display {
			menuItem.Check()
		} else {
			menuItem.Uncheck()
		}
	}
}
