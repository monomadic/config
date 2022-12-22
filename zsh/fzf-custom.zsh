CMD_LIST_RELATIVE_DIRS='cat ~/.marks'
CMD_DIRS_WORKSPACES='fd . ~/workspaces --extension workspace --follow'

function mark() {
	echo $PWD >> ~/.marks
}

function ls_marks() {
	cat ~/.marks
}

# list all workspaces (*.workspace)
function ls_workspaces() {
	fd . ~/workspaces --extension workspace --follow
}

function ls_all() {
	ls_marks
	ls_projects
	ls_workspaces
}

# list all projects (workspaces/*.workspace/**/*)
function ls_projects() {
	exa ~/workspaces/*.workspace/* --oneline --only-dirs --list-dirs
}

function ls_recursive() {
	fd --type d --strip-cwd-prefix --hidden --max-depth 5 --max-results 10000 --exclude node_modules --exclude .git --exclude target
}

function ls_hidden {
	exa --icons --group-directories-first
}

# fzf directory options
function fzf_dirs() {
	fzf --prompt 'cd  ' --layout=reverse --height 50% \
		--color=bg+:-1,fg:4,info:15,fg+:4,header:7,hl:5,hl+:5 \
		--header $'ctrl-[f:finder, w:workspace, o:bookmarks, r:relative, p:project, c:cancel]\n' \
		--info=hidden \
		--preview 'exa --tree --icons --level 2 {}' \
		--bind 'ctrl-w:change-prompt(workspaces > )+reload(fd . ~/workspaces --extension workspace --follow)' \
		--bind 'ctrl-o:change-prompt(bookmarks > )+reload(cat ~/.marks)' \
		--bind 'ctrl-p:change-prompt(projects > )+reload(exa ~/workspaces/*.workspace/* --oneline --only-dirs --list-dirs)' \
		--bind 'ctrl-f:execute-silent(open {1})' \
		--bind 'ctrl-r:change-prompt(relative > )+reload(fd --type d --strip-cwd-prefix --max-depth 5 --max-results 10000 --exclude target)' \
		"$@"
}

function fzf_edit() {
	files=$(ls_all|fzf_dirs)
	[[ -n "$files" ]] && cd "${files[@]}" && nvim . +"lua GoRoot()"
	zle && zle reset-prompt
}
zle -N fzf_edit

function fzf_cd() {
	files=$(ls_all|fzf_dirs)
	[[ -n "$files" ]] && cd "${files[@]}"
	zle && zle reset-prompt
}
zle -N fzf_cd

function fzf_cd_project() {
	files=$(ls_projects|fzf_dirs)
	[[ -n "$files" ]] && cd "${files[@]}"
	zle && zle reset-prompt
}
zle -N fzf_cd_project
