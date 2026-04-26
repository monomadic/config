//go:build !darwin

package systray

// SetTitleFont is implemented on macOS. Other platforms keep the default
// status item title font.
func SetTitleFont(size float64, bold bool) {}

// SetSystemSymbolIcon is implemented on macOS. Other platforms can use
// SetTemplateIcon with regular image bytes.
func SetSystemSymbolIcon(symbolName string) bool {
	return false
}
