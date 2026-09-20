package main

import (
	"bytes"
	"fmt"
	"os"
	"os/exec"
)

const appleScript = `
tell application "Finder"
	set selected_items to selection
	
	if selected_items is {} then
		try
			set target_item to target of front window as alias
		on error
			return
		end try
	else
		set target_item to item 1 of selected_items as alias
	end if
end tell

tell application "ForkLift"
	activate
	open target_item
end tell
`

func main() {
	cmd := exec.Command("/usr/bin/osascript", "-e", appleScript)

	var stderr bytes.Buffer
	cmd.Stderr = &stderr

	if err := cmd.Run(); err != nil {
		msg := stderr.String()
		if msg == "" {
			msg = err.Error()
		}

		notify("Finder → ForkLift failed", msg)
		os.Exit(1)
	}
}

func notify(title, message string) {
	script := fmt.Sprintf(
		`display notification %q with title %q`,
		message,
		title,
	)

	_ = exec.Command("/usr/bin/osascript", "-e", script).Run()
}
