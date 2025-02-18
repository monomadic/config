function fzf_create_template() {
  # sk --preview 'bat --style=numbers --color=always --line-range :500 {}'
  file=($(fd --full-path --type file --type symlink --base-directory ${TEMPLATE_BASE_DIR} |
    fzf --prompt 'template > ' --layout=reverse --preview "bat --style=numbers --color=always --line-range :500 ${BASE_DIRECTORY}/{1}" \
      --height 50% \
      --header $'ctrl-e:edit\n' \
      --bind "ctrl-e:execute:${EDITOR:-nvim} ${TEMPLATE_BASE_DIR}/{1}" \
      "$@"))
  [[ -n "$file" ]] && mkdir -p $(dirname $file) && cp -n -L ${TEMPLATE_BASE_DIR}/${file} ${PWD}/${file}
  echo "copied ${file}"
}
