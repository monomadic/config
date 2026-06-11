local mp = require "mp"

local function load_keybar_component()
    local path = mp.find_config_file("script-modules/keybar_component.lua")
    if not path then
        local source = debug.getinfo(1, "S").source
        local script_dir = source and source:match("^@(.+)/[^/]+$")
        path = script_dir and (script_dir .. "/../script-modules/keybar_component.lua")
    end
    return assert(loadfile(assert(path, "keybar_component.lua not found")))()
end

local keybar_component = load_keybar_component()

local overlay = mp.create_osd_overlay("ass-events")
overlay.z = 10

local keybar_enabled = true  -- what Tab toggles
local osd_ok = false -- derived from osd-level
local suppressors = {}
local CHAPTER_EDIT_SUPPRESSOR = "chapter-edit-mode"

-- PAN & SCAN state
local panscan = 0.0
local panscan_on = false
local last_panscan = 1.0
local PANSCAN_DEFAULT = 1.0

local auto_landscape = false
local metadata_enabled = true
local progress_visible = true
local stats_visible = false
local edge_fade_enabled = false
local skip_intros_enabled = false

-- prevents observe_property("panscan") from treating our own writes as "external user changes"
local applying_panscan = false

local function update_panscan_state(v)
    panscan = (type(v) == "number") and v or mp.get_property_number("panscan", 0.0)
    panscan_on = (panscan or 0.0) > 0.001
end
update_panscan_state()

local function apply_panscan_preference()
    -- what we *want* based on persistent state
    local want = 0.0
    if panscan_on then
        local v = (last_panscan and last_panscan > 0.001) and last_panscan or PANSCAN_DEFAULT
        want = v
    end

    applying_panscan = true
    mp.set_property_number("panscan", want)
    applying_panscan = false
end

-- RANDOM JUMP state (from the other script)
local rj_autojump_on = false
local rj_autoseek_on = false
local rj_delay = nil
local random_nav_script = "smart_playlist_nav/"

local command_equals = keybar_component.command_equals

local function matches_chapter_edit_toggle(entry)
    return command_equals("script-binding chapter-edit-mode/toggle")(entry)
        or command_equals("script-binding chapter_edit_mode/toggle")(entry)
end

local keybar_items = {
    {
        id = "menu",
        section = "shortcut",
        fallback = "ESC",
        prefer = { "ESC" },
        desc = " MENU",
        match = command_equals("script-binding show-menu"),
    },
    {
        id = "osd",
        section = "shortcut",
        fallback = "TAB",
        prefer = { "TAB" },
        desc = " OSD",
        match = command_equals("script-binding toggle-osd-full"),
    },
    {
        id = "chapter_edit",
        section = "shortcut",
        fallback = "c",
        prefer = { "c" },
        desc = " CHAPTERS",
        match = matches_chapter_edit_toggle,
    },
    {
        id = "panscan",
        section = "state",
        fallback = "p",
        prefer = { "p" },
        desc = function(badge)
            return " PAN " .. badge(panscan_on)
        end,
        match = command_equals("script-binding toggle-pan-scan"),
    },
    {
        id = "edge_fade",
        section = "state",
        fallback = "e",
        prefer = { "e" },
        desc = function(badge)
            return " EDGE FADE " .. badge(edge_fade_enabled)
        end,
        match = command_equals("script-binding toggle-edge-fade"),
    },
    {
        id = "progress",
        section = "state",
        fallback = "Ctrl+p",
        prefer = { "Ctrl+p", "4" },
        desc = function(badge)
            return " PROGRESS " .. badge(progress_visible)
        end,
        match = function(entry)
            return command_equals("script-binding progress_bar_minimal/toggle-progress")(entry)
                or command_equals("script-binding progress-bar-minimal/toggle-progress")(entry)
        end,
    },
    {
        id = "metadata",
        section = "state",
        fallback = "b",
        prefer = { "b" },
        desc = function(badge)
            return " META " .. badge(metadata_enabled)
        end,
        match = command_equals("script-message-to metadata toggle"),
    },
    {
        id = "stats",
        section = "state",
        fallback = "F7",
        prefer = { "F7" },
        desc = function(badge)
            return " STATS " .. badge(stats_visible)
        end,
        match = command_equals("script-binding toggle_stats"),
    },
    {
        id = "auto_landscape",
        section = "state",
        fallback = "2",
        prefer = { "2" },
        desc = function(badge)
            return " FORCE-HORZ " .. badge(auto_landscape)
        end,
        match = command_equals("script-binding toggle_force_landscape"),
    },
    {
        id = "skip_intros",
        section = "state",
        fallback = "3",
        prefer = { "3" },
        desc = function(badge)
            return " SKIP INTRO " .. badge(skip_intros_enabled)
        end,
        match = command_equals("script-binding toggle-skip-intros"),
    },
    {
        id = "auto_jump",
        section = "state",
        fallback = "1",
        prefer = { "1" },
        desc = function(badge, fmt_delay)
            return " AUTOJUMP " .. badge(rj_autojump_on) .. fmt_delay()
        end,
        match = command_equals("script-binding " .. random_nav_script .. "toggle_auto_jump"),
    },
}

local resolved_keys = {}

local function refresh_resolved_keys()
    local path = mp.find_config_file("input.conf")
    resolved_keys = keybar_component.resolve_item_keys(
        keybar_items,
        keybar_component.read_input_bindings(path)
    )
end

refresh_resolved_keys()

local function fmt_delay()
    if not rj_delay then return "" end
    local d = tonumber(rj_delay)
    if not d or d <= 0 then return "" end
    if d >= 60 then
        return (" %dm"):format(math.floor(d / 60 + 0.5))
    end
    return (" %ds"):format(math.floor(d + 0.5))
end

local function has_suppressor()
    for _, on in pairs(suppressors) do
        if on then
            return true
        end
    end
    return false
end

local render_bar

local function set_suppressor(name, on)
    if not name or name == "" then
        return
    end

    suppressors[name] = on or nil
    if render_bar then
        render_bar()
    end
end

local function build_bar(dim)
    dim = dim or mp.get_property_native("osd-dimensions")
    return keybar_component.build_bar(keybar_items, {
        dim = dim,
        resolved_keys = resolved_keys,
        context = {
            fmt_delay = fmt_delay,
        },
    })
end

mp.add_timeout(0, function()
    mp.commandv("script-message", "randjump-query")
    mp.commandv("script-message", "auto-landscape-query")
    mp.commandv("script-message", "metadata-query")
    mp.commandv("script-message", "progress-bar-query")
    mp.commandv("script-message", "realtime-stats-query")
    mp.commandv("script-message", "edge-fade-query")
    mp.commandv("script-message", "skip-intros-query")
    mp.commandv("script-message", "chapter-edit-mode-query")
end)

function render_bar()
    local visible = osd_ok and keybar_enabled and not has_suppressor()
    if not visible then
        overlay:remove()
        return
    end

    local dim = mp.get_property_native("osd-dimensions")
    if not dim or not dim.w or dim.w <= 0 or not dim.h or dim.h <= 0 then
        return
    end

    overlay.data  = build_bar(dim)
    overlay.res_x = dim.w
    overlay.res_y = dim.h
    overlay:update()
end

-- receive smart random navigation state updates
mp.register_script_message("keybar-randjump-state", function(a, f, d)
    rj_autojump_on = (a == "1" or a == "true")
    rj_autoseek_on = (f == "1" or f == "true")
    rj_delay = d
    render_bar()
end)

mp.register_event("file-loaded", function()
    -- mpv may re-apply defaults on file load; re-assert our preference after that
    mp.add_timeout(0, function()
        refresh_resolved_keys()
        apply_panscan_preference()
        update_panscan_state()
        render_bar()
        mp.commandv("script-message", "auto-landscape-query")
        mp.commandv("script-message", "metadata-query")
        mp.commandv("script-message", "progress-bar-query")
        mp.commandv("script-message", "realtime-stats-query")
        mp.commandv("script-message", "edge-fade-query")
        mp.commandv("script-message", "skip-intros-query")
        mp.commandv("script-message", "chapter-edit-mode-query")
    end)
end)

mp.observe_property("osd-dimensions", "native", function() render_bar() end)

mp.observe_property("osd-level", "number", function(_, v)
    osd_ok = (v and v > 0) or false
    render_bar()
end)

mp.observe_property("panscan", "number", function(_, v)
    if applying_panscan then
        update_panscan_state(v)
        render_bar()
        return
    end

    update_panscan_state(v)

    -- If something (user, another script, profile) sets panscan > 0, remember it.
    if (v or 0.0) > 0.001 then
        last_panscan = v
        panscan_on = true
    else
        panscan_on = false
    end

    render_bar()
end)

-- ask the other script for current state (best-effort)
mp.add_timeout(0, function()
    mp.commandv("script-message", "randjump-query")
end)

mp.register_script_message("keybar-toggle", function()
    keybar_enabled = not keybar_enabled
    render_bar()
end)

mp.register_script_message("keybar-suppress", function(name, state)
    set_suppressor(name, state == "yes" or state == "true" or state == "1")
end)

mp.register_script_message("chapter-edit-mode-state", function(state)
    set_suppressor(CHAPTER_EDIT_SUPPRESSOR, state == "yes" or state == "true" or state == "1")
end)

mp.register_script_message("metadata-state", function(state)
    metadata_enabled = (state == "yes")
    render_bar()
end)

mp.register_script_message("progress_bar_state", function(state)
    progress_visible = (state == "yes")
    render_bar()
end)

mp.register_script_message("realtime-stats-state", function(state)
    stats_visible = (state == "yes")
    render_bar()
end)

mp.register_script_message("edge-fade-state", function(state)
    edge_fade_enabled = (state == "yes")
    render_bar()
end)

mp.register_script_message("skip-intros-state", function(state)
    skip_intros_enabled = (state == "yes")
    render_bar()
end)

local function toggle_panscan()
    update_panscan_state()

    if panscan_on then
        last_panscan = panscan
        panscan_on = false
    else
        panscan_on = true
        if not last_panscan or last_panscan <= 0.001 then
            last_panscan = PANSCAN_DEFAULT
        end
    end

    apply_panscan_preference()
    render_bar()
end

mp.register_script_message("pan-scan-toggle", toggle_panscan)
mp.add_key_binding(nil, "toggle-pan-scan", toggle_panscan)

local OSD_FULL = 3
mp.add_key_binding(nil, "toggle-osd-full", function()
    local v = mp.get_property_number("osd-level", OSD_FULL)
    if v and v > 0 then
        mp.set_property_number("osd-level", 0)
    else
        mp.set_property_number("osd-level", OSD_FULL)
    end
end)

mp.register_script_message("auto_landscape_broadcast", function(state)
  auto_landscape = (state == "yes")
  render_bar()
end)
