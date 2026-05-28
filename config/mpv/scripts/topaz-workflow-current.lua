local utils = require "mp.utils"

local kitty_launch = "/Users/nom/.zsh/bin/kitty-launch"
local topaz_workflow = "/Users/nom/.zsh/bin/topaz-workflow"

local function absolute_media_path()
    local media_path = mp.get_property("path")

    if not media_path or media_path == "" then
        return nil, "No media loaded"
    end

    local proto = mp.get_property("protocol") or ""
    if proto ~= "" and proto ~= "file" then
        return nil, "Not a local file (" .. proto .. ")"
    end

    return utils.join_path(mp.get_property("working-directory") or "", media_path)
end

local function topaz_workflow_current_file()
    local abs, err = absolute_media_path()
    if not abs then
        mp.osd_message(err, 2)
        return
    end

    local info = utils.file_info(abs)
    if not info then
        mp.osd_message("File no longer exists:\n" .. abs, 3)
        mp.msg.warn("Topaz workflow failed, file missing: " .. abs)
        return
    end

    local directory = utils.split_path(abs)
    local result = utils.subprocess_detached({
        args = {
            kitty_launch,
            "--tab",
            "--hold",
            "--cwd", directory,
            "--title", " topaz workflow ",
            "--",
            topaz_workflow, abs,
        },
    })

    if result == false then
        mp.osd_message("Topaz workflow launch failed", 2)
        mp.msg.error("Failed to launch Topaz workflow for: " .. abs)
        return
    end

    mp.osd_message("Opening Topaz workflow", 1.5)
end

mp.add_key_binding(nil, "topaz_workflow_current_file", topaz_workflow_current_file)
