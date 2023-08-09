function joshuto-cd() {
	ID="$$"
	mkdir -p /tmp/$USER
	OUTPUT_FILE="/tmp/$USER/joshuto-cwd-$ID"
	env joshuto --output-file "$OUTPUT_FILE" $@
	exit_code=$?

	case "$exit_code" in
		# regular exit
		0)
			;;
		# output contains current directory
		101)
			JOSHUTO_CWD=$(cat "$OUTPUT_FILE")
			cd "$JOSHUTO_CWD"
			;;
		# output selected files
		102)
			;;
		*)
			echo "Exit code: $exit_code"
			;;
	esac
}; zle -N joshuto-cd;

function joshuto-wrapper() {
	zle kill-whole-line
	BUFFER="joshuto-cd"
	zle accept-line
	clear
	zle reset-prompt
}; zle -N joshuto-wrapper;
