BASE_DIRECTORY=${HOME}/.config/nvim/templates

function fzf_create_template() {
	# sk --preview 'bat --style=numbers --color=always --line-range :500 {}'
	file=($(fd --full-path --type file --type symlink --base-directory ${BASE_DIRECTORY} | \
		fzf --prompt 'template > ' --layout=reverse --preview "bat --style=numbers --color=always --line-range :500 ${BASE_DIRECTORY}/{1}" \
			--height 50% \
			--header $'ctrl-e:edit\n' \
			--bind "ctrl-e:execute:${EDITOR:-nvim} ${BASE_DIRECTORY}/{1}" \
		"$@"))
	[[ -n "$file" ]] && mkdir -p $(dirname $file) && cp -n -L ${BASE_DIRECTORY}/${file} ${PWD}/${file}
	echo "copied ${file}"
}
