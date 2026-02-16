-- Copy current media location to macOS clipboard (Cmd+C)
local function copy_current_path()
    local path = mp.get_property("path")
    if not path or path == "" then
        mp.osd_message("No media loaded")
        return
    end

    -- use subprocess to avoid shell escaping issues
    mp.command_native({
        name = "subprocess",
        playback_only = false,
        capture_stdout = false,
        args = { "pbcopy" },
        stdin_data = path
    })

    mp.osd_message("Copied to clipboard:\n" .. path, 1.5)
end

mp.add_key_binding("Meta+c", "copy-current-path", copy_current_path)
