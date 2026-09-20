on run
	set kittyPath to "/Applications/kitty.app/Contents/MacOS/kitty"
	set kittenPath to "/Applications/kitty.app/Contents/MacOS/kitten"
	set yaziPath to "/opt/homebrew/bin/yazi"
	set socketAddr to "unix:/tmp/kitty-" & (do shell script "id -un")
	
	tell application "Finder"
		set selectedItems to selection
		if selectedItems is {} then return
		
		set argList to {}
		repeat with anItem in selectedItems
			set end of argList to quoted form of POSIX path of (anItem as alias)
		end repeat
	end tell
	
	set yaziArgs to my joinList(argList, " ")
	
	set launchCmd to quoted form of kittenPath & Â
		" @ --to " & quoted form of socketAddr & Â
		" launch --type tab --cwd " & quoted form of (do shell script "pwd") & Â
		" -- " & quoted form of yaziPath & " " & yaziArgs
	
	try
		set winId to do shell script launchCmd
		
		do shell script quoted form of kittenPath & Â
			" @ --to " & quoted form of socketAddr & Â
			" focus-window --match " & quoted form of ("id:" & winId)
		
		tell application id "net.kovidgoyal.kitty" to activate
		
	on error errMsg number errNum
		set fallbackCmd to "nohup " & quoted form of kittyPath & Â
			" --listen-on " & quoted form of socketAddr & Â
			" -o allow_remote_control=socket-only" & Â
			" -- " & quoted form of yaziPath & " " & yaziArgs & Â
			" >/tmp/open-in-yazi.log 2>&1 &"
		
		do shell script fallbackCmd
		tell application id "net.kovidgoyal.kitty" to activate
	end try
end run

on joinList(theList, delimiter)
	set oldTID to AppleScript's text item delimiters
	set AppleScript's text item delimiters to delimiter
	set joinedText to theList as text
	set AppleScript's text item delimiters to oldTID
	return joinedText
end joinList