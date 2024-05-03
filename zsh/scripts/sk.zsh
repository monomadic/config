# search for a file recursively and cd into its container
sk-cd() {
    local file=$(fd . -t f | sk)

    if [ -n "$file" ]; then
        local dir=$(dirname "$file")
        cd "$dir"
				exa-ls
    fi
}

sk-rg() {
	sk -i -c "rg {} --color=always" --skip-to-pattern '[^/]*:' --ansi
}
