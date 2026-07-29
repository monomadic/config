package main

import (
	"fmt"
	"os/exec"
	"strings"
	"sync"
	"time"

	"github.com/caseymrm/menuet/v2"
)

const (
	batteryIcon    = "􀛨" // battery.100
	lowBatteryIcon = "􀛪" // battery.0
	boltIcon       = "􀋦" // bolt.fill

	barSegments         = 10
	barStrikeSize       = 20 // strike thickness and bar width scale with this font size
	lowBatteryThreshold = 10
	styleDefaultsKey    = "style"
)

type layoutStyle string

const (
	styleText          layoutStyle = "text"
	styleIconText      layoutStyle = "icon_text"
	styleBarText       layoutStyle = "bar_text"
	styleIconBar       layoutStyle = "icon_bar"
	stylePercentBar    layoutStyle = "percent_bar"
	styleBar           layoutStyle = "bar"
	styleBarPower      layoutStyle = "bar_power"
	styleSmartBar      layoutStyle = "smart_bar"
	styleSmartBarTimer layoutStyle = "smart_bar_timer"
)

func allLayoutStyles() []layoutStyle {
	return []layoutStyle{
		styleText,
		styleIconText,
		styleBarText,
		styleIconBar,
		stylePercentBar,
		styleBar,
		styleBarPower,
		styleSmartBar,
		styleSmartBarTimer,
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
	case styleIconBar:
		return "Icon and Bar"
	case stylePercentBar:
		return "Percentage and Bar"
	case styleBar:
		return "Bar"
	case styleBarPower:
		return "Bar and Power"
	case styleSmartBar:
		return "Smart Bar"
	case styleSmartBarTimer:
		return "Smart Bar and Timer"
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

var (
	infoMu   sync.RWMutex
	lastInfo batteryInfo
)

func main() {
	app := menuet.App()
	app.Name = "Battery Widget"
	app.Label = "com.jayu.battery-widget"
	app.Children = menuItems
	app.HideStartup() // installed as a LaunchAgent instead

	go func() {
		updateBattery()
		ticker := time.NewTicker(10 * time.Second)
		defer ticker.Stop()
		for range ticker.C {
			updateBattery()
		}
	}()

	app.RunApplication()
}

func currentStyle() layoutStyle {
	style := layoutStyle(menuet.Defaults().String(styleDefaultsKey))
	if !style.valid() {
		return styleSmartBar
	}
	return style
}

func setStyle(style layoutStyle) {
	menuet.Defaults().SetString(styleDefaultsKey, string(style))
	updateBattery()
	menuet.App().MenuChanged()
}

func updateBattery() {
	info, err := readBattery()
	if err != nil {
		fmt.Printf("error reading battery: %v\n", err)
		return
	}

	infoMu.Lock()
	lastInfo = info
	infoMu.Unlock()

	runs := titleRuns(info, currentStyle())
	// A strikethrough on the title's leading run doesn't render; pad with a
	// plain run when a bar leads.
	if len(runs) > 0 && runs[0].Strikethrough {
		runs = append([]menuet.TextRun{{Text: " "}}, runs...)
	}
	menuet.App().SetMenuState(&menuet.MenuState{Runs: runs})
}

func getInfo() batteryInfo {
	infoMu.RLock()
	defer infoMu.RUnlock()
	return lastInfo
}

// stateColor returns the smart-bar accent for the current state; the zero
// Color means "default menu bar text color".
func stateColor(info batteryInfo) menuet.Color {
	switch {
	case info.State == stateCharging:
		return menuet.SystemGreen
	case info.State == stateDischarging && info.Percent < lowBatteryThreshold:
		return menuet.SystemRed
	case info.LowPowerMode:
		return menuet.SystemYellow
	default:
		return menuet.Color{}
	}
}

// powerText renders battery power flow compactly: "8.4w" discharging,
// "+42w" charging.
func powerText(info batteryInfo) string {
	prefix := ""
	if info.State == stateCharging {
		prefix = "+"
	}
	if info.Watts >= 10 {
		return fmt.Sprintf("%s%.0fw", prefix, info.Watts)
	}
	return fmt.Sprintf("%s%.1fw", prefix, info.Watts)
}

func percentText(info batteryInfo) string {
	return fmt.Sprintf("%d%%", info.Percent)
}

func statusIcon(info batteryInfo) string {
	switch {
	case info.State == stateCharging:
		return boltIcon
	case info.Percent < lowBatteryThreshold:
		return lowBatteryIcon
	default:
		return batteryIcon
	}
}

// barRuns draws the progress bar as strikethrough space runs: the strike
// renders as one thin continuous line, vertically centered against the
// neighboring text. Glyph-based bars (━, █) leave per-cell gaps or seams,
// and run backgrounds always fill the whole line height. A zero fill color
// means the default label color.
func barRuns(percent int, fill menuet.Color) []menuet.TextRun {
	filled := (percent*barSegments + 50) / 100
	if filled < 0 {
		filled = 0
	}
	if filled > barSegments {
		filled = barSegments
	}
	if fill.IsZero() {
		// A zero StrikethroughColor resolves to transparent on the ObjC
		// side, not "follow foreground" — always send an explicit color.
		fill = menuet.LabelPrimary
	}

	bar := func(cells int, color menuet.Color) menuet.TextRun {
		return menuet.TextRun{
			Text:               strings.Repeat(" ", cells),
			FontSize:           barStrikeSize,
			Monospaced:         true,
			Strikethrough:      true,
			StrikethroughColor: color,
		}
	}

	runs := []menuet.TextRun{}
	if filled > 0 {
		runs = append(runs, bar(filled, fill))
	}
	if filled < barSegments {
		runs = append(runs, bar(barSegments-filled, menuet.LabelQuaternary))
	}
	return runs
}

func titleRuns(info batteryInfo, style layoutStyle) []menuet.TextRun {
	plain := func(text string) menuet.TextRun { return menuet.TextRun{Text: text} }

	switch style {
	case styleText:
		return []menuet.TextRun{plain(percentText(info))}
	case styleIconText:
		return []menuet.TextRun{plain(statusIcon(info) + " " + percentText(info))}
	case styleBarText:
		return append(barRuns(info.Percent, menuet.Color{}), plain(" "+percentText(info)))
	case styleIconBar:
		return append([]menuet.TextRun{{Text: boltIcon + " ", Color: menuet.LabelSecondary}},
			barRuns(info.Percent, menuet.Color{})...)
	case stylePercentBar:
		return append([]menuet.TextRun{plain(percentText(info) + " ")},
			barRuns(info.Percent, menuet.Color{})...)
	case styleBar:
		return barRuns(info.Percent, menuet.Color{})
	case styleBarPower:
		return append(barRuns(info.Percent, menuet.Color{}), plain(" "+powerText(info)))
	case styleSmartBar, styleSmartBarTimer:
		return smartBarRuns(info, style == styleSmartBarTimer)
	default:
		return []menuet.TextRun{plain(percentText(info))}
	}
}

func smartBarRuns(info batteryInfo, withTimer bool) []menuet.TextRun {
	accent := stateColor(info)

	runs := []menuet.TextRun{}
	if info.State == stateCharging {
		runs = append(runs, menuet.TextRun{Text: boltIcon + " ", Color: accent})
	}
	runs = append(runs, barRuns(info.Percent, accent)...)
	runs = append(runs, menuet.TextRun{Text: " " + powerText(info), Color: accent})

	if withTimer && info.TimeRemaining != "" {
		runs = append(runs,
			menuet.TextRun{Text: " · ", Color: menuet.LabelTertiary},
			menuet.TextRun{Text: info.TimeRemaining, Color: menuet.LabelSecondary, Monospaced: true},
		)
	}
	return runs
}

func menuItems() []menuet.MenuItem {
	info := getInfo()

	statusLine := fmt.Sprintf("Battery: %d%%", info.Percent)
	switch {
	case info.State == stateCharging && info.TimeRemaining != "":
		statusLine += fmt.Sprintf(" — %s until full", info.TimeRemaining)
	case info.State == stateDischarging && info.TimeRemaining != "":
		statusLine += fmt.Sprintf(" — %s remaining", info.TimeRemaining)
	case info.State == stateIdle && info.OnAC:
		statusLine += " — charged"
	}

	source := "battery"
	if info.OnAC {
		source = "power adapter"
	}

	lowPowerLabel := "Low Power Mode: Off"
	if info.LowPowerMode {
		lowPowerLabel = "Low Power Mode: On"
	}

	items := []menuet.MenuItem{
		menuet.Regular{Text: statusLine},
		menuet.Regular{Text: fmt.Sprintf("Power draw: %s (%s)", powerText(info), source)},
		menuet.Regular{Text: fmt.Sprintf("Health: %d%% · %d cycles", info.HealthPercent, info.CycleCount)},
		menuet.Separator{},
		menuet.Regular{Text: "Style", Children: styleItems},
		menuet.Separator{},
		menuet.Regular{Text: lowPowerLabel, Clicked: toggleLowPowerMode},
	}
	return items
}

func styleItems() []menuet.MenuItem {
	current := currentStyle()
	items := []menuet.MenuItem{}
	for _, style := range allLayoutStyles() {
		style := style
		items = append(items, menuet.Regular{
			Text:    style.label(),
			State:   style == current,
			Clicked: func() { setStyle(style) },
		})
	}
	return items
}

func toggleLowPowerMode() {
	target := "1"
	if getInfo().LowPowerMode {
		target = "0"
	}
	script := fmt.Sprintf(`do shell script "pmset -a lowpowermode %s" with administrator privileges`, target)
	if err := exec.Command("osascript", "-e", script).Run(); err != nil {
		fmt.Printf("error toggling low power mode: %v\n", err)
	}
	updateBattery()
	menuet.App().MenuChanged()
}
