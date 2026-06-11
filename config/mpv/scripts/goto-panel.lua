-- goto-panel.lua - "go to" chord panel, bound to "g" in input.conf
-- (script-message toggle-goto-panel).
local mp = require "mp"

local function load_chord_panel()
    local path = mp.find_config_file("script-modules/chord_panel.lua")
    if not path then
        local source = debug.getinfo(1, "S").source
        local script_dir = source and source:match("^@(.+)/[^/]+$")
        path = script_dir and (script_dir .. "/../script-modules/chord_panel.lua")
    end
    return assert(loadfile(assert(path, "chord_panel.lua not found")))()
end

local chord_panel = load_chord_panel()

local function seek_percent(pct)
    mp.commandv("seek", tostring(pct), "absolute-percent+exact")
end

local function play_index(index)
    mp.commandv("playlist-play-index", tostring(index))
end

local panel = chord_panel.new({
    name = "goto-panel",
    title = "go to",
    actions = {
        {
            key = "h",
            label = "start of current track",
            fn = function() seek_percent(0) end,
        },
        {
            key = "H",
            label = "first track in playlist",
            fn = function() play_index(0) end,
        },
        {
            key = "L",
            label = "last track in playlist",
            fn = function()
                local count = mp.get_property_number("playlist-count", 0)
                if count > 0 then
                    play_index(count - 1)
                end
            end,
        },
        {
            key = "r",
            label = "random track in playlist",
            fn = function()
                mp.commandv("script-binding", "smart_playlist_nav/playlist-random-track")
            end,
        },
        {
            keys = { "0", "1", "2", "3", "4", "5", "6", "7", "8", "9" },
            key_label = "0-9",
            label = "seek to 0%-90% of current track",
            fn = function(key) seek_percent(tonumber(key) * 10) end,
        },
    },
})

mp.add_key_binding(nil, "toggle-goto-panel", function() panel:toggle() end)
mp.register_script_message("toggle-goto-panel", function() panel:toggle() end)
