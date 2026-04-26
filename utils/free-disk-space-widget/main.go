package main

import (
	"bytes"
	"fmt"
	"image"
	"image/color"
	"image/png"
	"math"
	"os/exec"
	"regexp"
	"time"

	"github.com/getlantern/systray"
)

func main() {
	systray.Run(onReady, onExit)
}

func formatBytesToGb(bytes int64) string {
	freeGB := int64(math.Round(float64(bytes) / 1024 / 1024 / 1024))
	return fmt.Sprintf("%d GB", freeGB)
}

func diskIcon() []byte {
	img := image.NewRGBA(image.Rect(0, 0, 18, 18))
	ink := color.RGBA{A: 255}

	for y := 5; y <= 13; y++ {
		for x := 2; x <= 15; x++ {
			onOuterEdge := y == 5 || y == 13 || x == 2 || x == 15
			onInnerLine := y == 10 && x >= 4 && x <= 13
			if onOuterEdge || onInnerLine {
				img.Set(x, y, ink)
			}
		}
	}

	img.Set(12, 12, ink)
	img.Set(13, 12, ink)

	var buf bytes.Buffer
	if err := png.Encode(&buf, img); err != nil {
		return nil
	}
	return buf.Bytes()
}

func updateFreeSpace() {
	process := exec.Command("diskutil", "info", "/")
	process.Wait()
	output, err := process.CombinedOutput()

	if err != nil {
		fmt.Printf("error running diskutil: %v\nOutput: %s", err, string(output))
		return
	}

	re := regexp.MustCompile(`Container Free Space:\s+[\d.]+\s+GB\s+\((\d+)\s+Bytes\)`)
	matches := re.FindStringSubmatch(string(output))
	if len(matches) < 2 {
		fmt.Println("could not find free space in diskutil output")
		return
	}

	var freeBytes int64
	_, err = fmt.Sscanf(matches[1], "%d", &freeBytes)
	if err != nil {
		fmt.Printf("error parsing free space: %v", err)
		return
	}

	freeGB := formatBytesToGb(freeBytes)
	systray.SetTitle(freeGB)
	tooltip := fmt.Sprintf("Free disk space is %s (%d Bytes)", freeGB, freeBytes)
	systray.SetTooltip(tooltip)

}

func onReady() {

	if !systray.SetSystemSymbolIcon("internaldrive") {
		icon := diskIcon()
		systray.SetTemplateIcon(icon, icon)
	}
	systray.SetTitleFont(12, true)
	systray.SetTitle("init")
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
