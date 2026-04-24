local mp = require("mp")

local SHADER = "shaders/edge-fade.glsl"
local OSD_SECS = 1.2

local enabled = false

local function shader_path()
    return mp.find_config_file(SHADER)
end

local function broadcast_state()
    mp.commandv("script-message", "edge-fade-state", enabled and "yes" or "no")
end

local function remove_shader(path)
    if path then
        pcall(mp.commandv, "change-list", "glsl-shaders", "remove", path)
    end
end

local function apply_state(show_osd)
    local path = shader_path()
    if not path then
        enabled = false
        broadcast_state()
        mp.osd_message("Edge fade: shader not found", OSD_SECS)
        return
    end

    remove_shader(path)

    if enabled then
        pcall(mp.commandv, "change-list", "glsl-shaders", "append", path)
    end

    broadcast_state()

    if show_osd then
        mp.osd_message("Edge fade: " .. (enabled and "ON" or "OFF"), OSD_SECS)
    end
end

local function toggle()
    enabled = not enabled
    apply_state(true)
end

mp.register_script_message("edge-fade-query", broadcast_state)
mp.add_key_binding(nil, "toggle-edge-fade", toggle)

mp.register_event("file-loaded", function()
    apply_state(false)
end)

mp.add_timeout(0, broadcast_state)
