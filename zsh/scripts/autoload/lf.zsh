# lf directory change
function lfcd() {
	local tmp="$(mktemp)"
	lf -last-dir-path="$tmp" "$@"
	if [ -f "$tmp" ]; then
			dir="$(cat "$tmp")"
			rm -f "$tmp"
			if [ -d "$dir" ]; then
					if [ "$dir" != "$(pwd)" ]; then
							cd "$dir"
							clear
							exa --icons --group-directories-first
							echo
							zle && zle reset-prompt
					fi
			fi
	fi
}
zle -N lfcd;
