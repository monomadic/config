# search for a file recursively and cd into its container
fcd() {
    local file=$(fd . -t f | fzf)

    if [ -n "$file" ]; then
        local dir=$(dirname "$file")
        cd "$dir"
    fi
}
