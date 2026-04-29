local mp = require "mp"

local overlay = mp.create_osd_overlay("ass-events")
overlay.z = 10

local keybar_enabled = true  -- what Tab toggles
local osd_ok = false -- derived from osd-level

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

local function trim(s)
    return (s or ""):match("^%s*(.-)%s*$")
end

local function normalize_command(command)
    command = trim(command)
    command = command:gsub("^script_binding%s+", "script-binding ")
    command = command:gsub("%s+", " ")
    return command
end

local function normalize_binding_key(binding)
    local parts = {}

    for part in tostring(binding or ""):gmatch("[^+]+") do
        table.insert(parts, trim(part))
    end

    for i = 1, math.max(#parts - 1, 0) do
        local lower = parts[i]:lower()
        if lower == "meta" then
            parts[i] = "Meta"
        elseif lower == "ctrl" then
            parts[i] = "Ctrl"
        elseif lower == "alt" then
            parts[i] = "Alt"
        elseif lower == "shift" then
            parts[i] = "Shift"
        end
    end

    if #parts > 0 then
        local lower = parts[#parts]:lower()
        local special_keys = {
            esc = "ESC",
            tab = "TAB",
            space = "SPACE",
            enter = "ENTER",
            left = "LEFT",
            right = "RIGHT",
        }
        parts[#parts] = special_keys[lower] or parts[#parts]
    end

    return table.concat(parts, "+")
end

local function format_key_label(binding)
    local mods = {
        Meta = "⌘",
        Ctrl = "⌃",
        Alt = "⌥",
        Shift = "⇧",
    }
    local keys = {
        ESC = "Esc",
        TAB = "Tab",
        SPACE = "Space",
        ENTER = "Enter",
        LEFT = "←",
        RIGHT = "→",
    }
    local parts = {}
    local out = {}

    binding = normalize_binding_key(binding)

    for part in binding:gmatch("[^+]+") do
        table.insert(parts, part)
    end

    for _, part in ipairs(parts) do
        table.insert(out, mods[part] or keys[part] or part)
    end

    return table.concat(out)
end

local function command_equals(target)
    target = normalize_command(target)
    return function(entry)
        return entry.command == target
    end
end

local function command_prefix(prefix)
    prefix = normalize_command(prefix)
    return function(entry)
        return entry.command:sub(1, #prefix) == prefix
    end
end

local function command_contains(...)
    local needles = { ... }
    return function(entry)
        for _, needle in ipairs(needles) do
            if not entry.command:find(needle, 1, true) then
                return false
            end
        end
        return true
    end
end

local keybar_items = {
    {
        id = "menu",
        section = "shortcut",
        fallback = "ESC",
        prefer = { "ESC" },
        desc = " Menu",
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
        id = "dir",
        section = "shortcut",
        fallback = "Meta+d",
        prefer = { "Meta+d" },
        desc = " Open Dir",
        match = command_equals("script-binding replace-playlist"),
    },
    {
        id = "sort",
        section = "shortcut",
        fallback = "Ctrl+s",
        prefer = { "Ctrl+s" },
        desc = " Sort",
        match = command_equals("script-binding sort_playlist_by_mtime"),
    },
    {
        id = "expand",
        section = "shortcut",
        fallback = "Ctrl+d",
        prefer = { "Ctrl+d" },
        desc = " Expand",
        match = command_equals("script-binding expand_playlist_dirs"),
    },
    {
        id = "panscan",
        section = "state",
        fallback = "p",
        prefer = { "p" },
        desc = function(badge)
            return " Pan " .. badge(panscan_on)
        end,
        match = command_equals("script-binding toggle-pan-scan"),
    },
    {
        id = "edge_fade",
        section = "state",
        fallback = "e",
        prefer = { "e" },
        desc = function(badge)
            return " Edge Fade " .. badge(edge_fade_enabled)
        end,
        match = command_equals("script-binding toggle-edge-fade"),
    },
    {
        id = "progress",
        section = "state",
        fallback = "Ctrl+p",
        prefer = { "Ctrl+p", "4" },
        desc = function(badge)
            return " Progress " .. badge(progress_visible)
        end,
        match = command_equals("script-binding progress-bar-minimal/toggle-progress"),
    },
    {
        id = "metadata",
        section = "state",
        fallback = "b",
        prefer = { "b" },
        desc = function(badge)
            return " Meta " .. badge(metadata_enabled)
        end,
        match = command_equals("script-message-to metadata toggle"),
    },
    {
        id = "stats",
        section = "state",
        fallback = "F7",
        prefer = { "F7" },
        desc = function(badge)
            return " Stats " .. badge(stats_visible)
        end,
        match = command_equals("script-binding toggle_stats"),
    },
    {
        id = "auto_landscape",
        section = "state",
        fallback = "2",
        prefer = { "2" },
        desc = function(badge)
            return " Auto-Land " .. badge(auto_landscape)
        end,
        match = command_equals("script-binding toggle_force_landscape"),
    },
    {
        id = "skip_intros",
        section = "state",
        fallback = "3",
        prefer = { "3" },
        desc = function(badge)
            return " Skip Intro " .. badge(skip_intros_enabled)
        end,
        match = command_equals("script-binding toggle-skip-intros"),
    },
    {
        id = "auto_jump",
        section = "state",
        fallback = "1",
        prefer = { "1" },
        desc = function(badge, fmt_delay)
            return " Auto Jump " .. badge(rj_autojump_on) .. fmt_delay()
        end,
        match = command_equals("script-binding toggle_auto_jump"),
    },
}

local resolved_keys = {}

local function read_input_bindings()
    local path = mp.find_config_file("input.conf")
    local bindings = {}

    if not path then
        return bindings
    end

    local file = io.open(path, "r")
    if not file then
        return bindings
    end

    for line in file:lines() do
        local cleaned = trim(line)
        if cleaned ~= "" and not cleaned:match("^#") then
            local key, command = cleaned:match("^(%S+)%s+(.+)$")
            if key and command then
                table.insert(bindings, {
                    key = normalize_binding_key(key),
                    command = normalize_command(command),
                })
            end
        end
    end

    file:close()
    return bindings
end

local function resolve_item_key(item, bindings)
    local matches = {}

    for _, entry in ipairs(bindings) do
        if item.match(entry) then
            table.insert(matches, entry.key)
        end
    end

    if item.prefer then
        for _, preferred in ipairs(item.prefer) do
            preferred = normalize_binding_key(preferred)
            for _, candidate in ipairs(matches) do
                if candidate == preferred then
                    return format_key_label(candidate)
                end
            end
        end
    end

    if #matches > 0 then
        return format_key_label(matches[1])
    end

    return format_key_label(item.fallback)
end

local function refresh_resolved_keys()
    local bindings = read_input_bindings()

    for _, item in ipairs(keybar_items) do
        resolved_keys[item.id] = resolve_item_key(item, bindings)
    end
end

refresh_resolved_keys()

local function build_bar(dim)
    dim = dim or mp.get_property_native("osd-dimensions")
    local w = (dim and dim.w) or 1280
    local h = (dim and dim.h) or 720

    local scale = 9
    local progress_h = 5
    local text_y = h - progress_h
    local key_color  = "{\\1c&H9CFF00&}"
    local text_color = "{\\1c&HFFFFFF&}"
    local chip_color = "{\\1c&H181818&\\alpha&H00&}"
    local sep_color  = "{\\1c&H6A6A6A&}"

    local function badge(on)
        local c = on and "{\\1c&H00FFFF&}" or "{\\1c&H777777&}"
        return c .. (on and "ON" or "OFF") .. text_color
    end

    local function fmt_delay()
        if not rj_delay then return "" end
        local d = tonumber(rj_delay)
        if not d or d <= 0 then return "" end
        if d >= 60 then
            return (" %dm"):format(math.floor(d / 60 + 0.5))
        end
        return (" %ds"):format(math.floor(d + 0.5))
    end

    local function key(label, desc)
        return chip_color .. "  " .. key_color .. label .. text_color .. desc .. "  "
    end

    local function desc(item)
        if type(item.desc) == "function" then
            return item.desc(badge, fmt_delay)
        end
        return item.desc
    end

    local s = ("{\\an2\\pos(%d,%d)\\bord0\\shad0\\scale%d}"):format(
        math.floor(w / 2),
        text_y,
        scale
    )

    local groups = {}
    local current_group = nil
    local last_section = nil

    for _, item in ipairs(keybar_items) do
        if item.section ~= last_section then
            current_group = {}
            table.insert(groups, current_group)
            last_section = item.section
        end
        table.insert(current_group, key(resolved_keys[item.id] or format_key_label(item.fallback), desc(item)))
    end

    local rendered_groups = {}
    for _, group in ipairs(groups) do
        table.insert(rendered_groups, table.concat(group, sep_color .. "|" .. text_color))
    end

    return s .. table.concat(rendered_groups, sep_color .. "  |  " .. text_color)
end

mp.add_timeout(0, function()
    mp.commandv("script-message", "randjump-query")
    mp.commandv("script-message", "auto-landscape-query")
    mp.commandv("script-message", "metadata-query")
    mp.commandv("script-message", "progress-bar-query")
    mp.commandv("script-message", "realtime-stats-query")
    mp.commandv("script-message", "edge-fade-query")
    mp.commandv("script-message", "skip-intros-query")
end)

local function render_bar()
    local visible = osd_ok and keybar_enabled
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

-- receive random-jump state updates
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
