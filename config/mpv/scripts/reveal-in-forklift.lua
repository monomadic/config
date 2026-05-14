local utils = require "mp.utils"

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

local function reveal_in_forklift()
    local abs, err = absolute_media_path()
    if not abs then
        mp.osd_message(err, 2)
        return
    end

    local info = utils.file_info(abs)
    if not info then
        mp.osd_message("File no longer exists:\n" .. abs, 3)
        mp.msg.warn("Reveal in ForkLift failed, file missing: " .. abs)
        return
    end

    local result = utils.subprocess({
        args = {
            "/usr/bin/osascript",
            "-e", "on run argv",
            "-e", "set targetPath to item 1 of argv",
            "-e", "tell application id \"com.binarynights.ForkLift\" to reveal path targetPath",
            "-e", "end run",
            abs,
        },
        cancellable = false,
    })

    if result.status ~= 0 then
        local details = result.stderr or result.error or "unknown"
        mp.osd_message("Reveal in ForkLift failed", 2)
        mp.msg.error("osascript reveal failed: " .. details)
        return
    end

    mp.osd_message("Revealed in ForkLift", 1.5)
end

mp.add_key_binding(nil, "reveal_in_forklift", reveal_in_forklift)
