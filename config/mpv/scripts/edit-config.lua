-- edit-config.lua - open mpv.conf in helix inside a new kitty window.
-- Bound to Meta+, (⌘,) in input.conf via `script-binding edit_config_in_helix`.
-- Requires `macos-menu-shortcuts=no` in mpv.conf, otherwise the native macOS
-- menu bar swallows ⌘, (it opens the config in the default GUI editor) before
-- input.conf ever sees the key.
local mp = require "mp"
local utils = require "mp.utils"

local KITTY_LAUNCH = "/Users/nom/.zsh/bin/kitty-launch"
local EDITOR = "hx"

local function config_path()
    -- Use the resolved mpv.conf if it exists; otherwise fall back to its
    -- expected location so the editor can create it.
    local path = mp.find_config_file("mpv.conf")
    if not path then
        path = mp.command_native({ "expand-path", "~~/mpv.conf" })
    end
    return path
end

local function edit_config()
    local path = config_path()
    if not path or path == "" then
        mp.osd_message("Could not resolve mpv.conf path", 3)
        mp.msg.error("edit-config: unable to resolve mpv.conf path")
        return
    end

    local directory = utils.split_path(path)
    local result = utils.subprocess_detached({
        args = {
            KITTY_LAUNCH,
            "--os-window",
            "--cwd", directory,
            "--title", " mpv config ",
            "--",
            EDITOR, path,
        },
    })

    if result == false then
        mp.osd_message("Failed to open editor", 2)
        mp.msg.error("edit-config: failed to launch " .. EDITOR .. " for: " .. path)
        return
    end

    mp.osd_message("Editing mpv.conf", 1.5)
end

mp.add_key_binding(nil, "edit_config_in_helix", edit_config)
mp.register_script_message("edit-config-in-helix", edit_config)
