local mp = require "mp"

local SKIP_SECONDS = 7
local MIN_READAHEAD_SECONDS = 10
local enabled = false
local saved_readahead = nil

local function broadcast_state()
    mp.commandv("script-message", "skip-intros-state", enabled and "yes" or "no")
end

local function osd_message(message)
    local osd_level = mp.get_property_number("osd-level", 0)
    if osd_level and osd_level > 0 then
        mp.osd_message(message, 1.5)
    end
end

local function apply_cache_floor()
    if not enabled then
        return
    end

    local current = mp.get_property_number("demuxer-readahead-secs", 0)
    if current and current >= MIN_READAHEAD_SECONDS then
        return
    end

    if saved_readahead == nil then
        saved_readahead = current
    end

    mp.set_property_number("demuxer-readahead-secs", MIN_READAHEAD_SECONDS)
end

local function restore_cache_floor()
    if saved_readahead ~= nil then
        mp.set_property_number("demuxer-readahead-secs", saved_readahead)
        saved_readahead = nil
    end
end

local function skip_intro()
    if not enabled then
        return
    end

    apply_cache_floor()

    local duration = mp.get_property_number("duration", 0)
    if duration and duration > SKIP_SECONDS then
        mp.commandv("seek", tostring(SKIP_SECONDS), "absolute", "exact")
    end
end

local function toggle()
    enabled = not enabled
    if enabled then
        apply_cache_floor()
    else
        restore_cache_floor()
    end
    broadcast_state()
    osd_message("Skip intros: " .. (enabled and "ON" or "OFF"))
end

mp.register_event("file-loaded", function()
    mp.add_timeout(0, skip_intro)
end)

mp.register_script_message("toggle", toggle)
mp.register_script_message("skip-intros-query", broadcast_state)
mp.add_key_binding(nil, "toggle-skip-intros", toggle)
mp.add_timeout(0, broadcast_state)
