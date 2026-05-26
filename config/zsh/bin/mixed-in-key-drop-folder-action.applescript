on adding folder items to this_folder after receiving added_items
	set command_text to quoted form of "/Users/nom/config/config/zsh/bin/mixed-in-key-drop" & " send"

	repeat with added_item in added_items
		set command_text to command_text & " " & quoted form of POSIX path of added_item
	end repeat

	do shell script command_text
end adding folder items to
