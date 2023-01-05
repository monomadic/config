# FZF jumps
#
# - https://github-wiki-see.page/m/junegunn/fzf/wiki/Color-schemes
#

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
	fd --type d --strip-cwd-prefix --max-depth 5 --max-results 10000 --exclude node_modules --exclude .git --exclude target
}

# 2 levels deep, immediate first
function ls_relative() {
	fd --type d --strip-cwd-prefix --max-depth 1
	fd --type d --strip-cwd-prefix --exact-depth 2 --max-results 10000
	# fd --type d --strip-cwd-prefix --exact-depth 3 --max-results 10000
}

function ls_hidden {
	exa --icons --group-directories-first
}

# ripgrep
function fzf-rg {
	RG_PREFIX="rg --column --line-number --no-heading --color=always --smart-case "
  fzf --bind "change:reload:$RG_PREFIX {q} || true" --ansi --disabled
}

# fzf directory options
function fzf_dirs() {
	fzf --prompt 'cd  ' --layout=reverse --height 60% \
		--color=bg+:-1,fg:4,info:15,fg+:5,header:7,hl:5,hl+:5 \
		--header $'ctrl-[f:finder, w:workspace, o:bookmarks, r:relative, p:project, c:cancel]\n' \
		--info=hidden \
		--pointer=' ' \
		--preview 'exa --tree --icons --level 2 {}' \
		--bind 'ctrl-f:execute-silent(open {1})' \
		--bind 'ctrl-o:change-prompt(bookmarks > )+reload(cat ~/.marks)' \
		--bind 'ctrl-p:change-prompt(projects > )+reload(exa ~/workspaces/*.workspace/* --oneline --only-dirs --list-dirs)' \
		--bind 'ctrl-r:change-prompt(relative > )+reload(fd --type d --strip-cwd-prefix --max-depth 1 && fd --type d --strip-cwd-prefix --exact-depth 2 --max-results 10000)' \
		--bind 'ctrl-w:change-prompt(workspaces > )+reload(fd . ~/workspaces --extension workspace --follow)' \
		"$@"
}

function fzf_edit() {
	files=$(ls_all|fzf_dirs)
	[[ -n "$files" ]] && cd "${files[@]}" && nvim . -c "lua GoRoot()"
	zle && zle reset-prompt
}
zle -N fzf_edit

function fzf_cd() {
	files=$(ls_all|fzf_dirs)
	[[ -n "$files" ]] && cd "${files[@]}"
	clear

	printf ' %s/\n\n' "${PWD##*/}"
	exa --icons --group-directories-first --all --no-time --no-permissions --no-user -l --ignore-glob '.DS_Store' --color=always |head -n 15
	echo
	zle && zle reset-prompt
}
zle -N fzf_cd

function fzf_cd_project() {
	files=$(ls_projects|fzf_dirs)
	[[ -n "$files" ]] && cd "${files[@]}"
	zle && zle reset-prompt
}
zle -N fzf_cd_project
