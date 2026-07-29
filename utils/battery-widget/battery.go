package main

import (
	"fmt"
	"math"
	"os/exec"
	"regexp"
	"strconv"
	"strings"
)

type batteryState int

const (
	stateDischarging batteryState = iota
	stateCharging
	stateIdle // on AC, full or not charging
)

type batteryInfo struct {
	Percent       int
	State         batteryState
	TimeRemaining string // "3:12", empty when macOS has no estimate
	Watts         float64 // magnitude of battery power flow
	HealthPercent int
	CycleCount    int
	LowPowerMode  bool
	OnAC          bool
}

var (
	pmsetBattRe = regexp.MustCompile(`(\d+)%; ([^;]+);`)
	pmsetTimeRe = regexp.MustCompile(`(\d+:\d+) remaining`)
	ioregKeyRe  = regexp.MustCompile(`"(\w+)" = (-?\d+)`)
)

func readBattery() (batteryInfo, error) {
	info := batteryInfo{}

	battOut, err := exec.Command("pmset", "-g", "batt").Output()
	if err != nil {
		return info, fmt.Errorf("pmset -g batt: %w", err)
	}
	batt := string(battOut)

	info.OnAC = strings.Contains(batt, "'AC Power'")

	match := pmsetBattRe.FindStringSubmatch(batt)
	if match == nil {
		return info, fmt.Errorf("no battery found in pmset output")
	}
	info.Percent, _ = strconv.Atoi(match[1])

	// "not charging" also contains "charging", so match the exact state word.
	switch state := strings.TrimSpace(match[2]); {
	case state == "discharging":
		info.State = stateDischarging
	case state == "charging" || state == "finishing charge":
		info.State = stateCharging
	default:
		info.State = stateIdle
	}

	if timeMatch := pmsetTimeRe.FindStringSubmatch(batt); timeMatch != nil {
		info.TimeRemaining = timeMatch[1]
	}

	ioregOut, err := exec.Command("ioreg", "-rn", "AppleSmartBattery").Output()
	if err != nil {
		return info, fmt.Errorf("ioreg AppleSmartBattery: %w", err)
	}
	values := map[string]int64{}
	for _, kv := range ioregKeyRe.FindAllStringSubmatch(string(ioregOut), -1) {
		values[kv[1]] = parseIoregInt(kv[2])
	}

	info.CycleCount = int(values["CycleCount"])
	info.Watts = math.Abs(float64(values["Amperage"]) * float64(values["Voltage"]) / 1e6)

	// Apple silicon reports NominalChargeCapacity/DesignCapacity in mAh;
	// fall back to AppleRawMaxCapacity for older machines.
	maxCapacity := values["NominalChargeCapacity"]
	if maxCapacity == 0 {
		maxCapacity = values["AppleRawMaxCapacity"]
	}
	if design := values["DesignCapacity"]; design > 0 && maxCapacity > 0 {
		info.HealthPercent = int(math.Round(100 * float64(maxCapacity) / float64(design)))
	}

	if psOut, err := exec.Command("pmset", "-g").Output(); err == nil {
		for _, line := range strings.Split(string(psOut), "\n") {
			fields := strings.Fields(line)
			if len(fields) == 2 && fields[0] == "lowpowermode" {
				info.LowPowerMode = fields[1] == "1"
			}
		}
	}

	return info, nil
}

// parseIoregInt handles ioreg printing negative numbers (e.g. Amperage while
// discharging) as wrapped unsigned 64-bit decimals.
func parseIoregInt(text string) int64 {
	if value, err := strconv.ParseInt(text, 10, 64); err == nil {
		return value
	}
	if value, err := strconv.ParseUint(text, 10, 64); err == nil {
		return int64(value)
	}
	return 0
}
