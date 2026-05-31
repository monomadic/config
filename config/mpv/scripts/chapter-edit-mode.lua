local mp = require "mp"
local assdraw = require "mp.assdraw"

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

local bar_overlay = mp.create_osd_overlay("ass-events")
bar_overlay.z = 210

local progress_overlay = mp.create_osd_overlay("ass-events")
progress_overlay.z = 205

local enabled = false
local resolved_keys = {}

local DELETE_THRESHOLD_SECONDS = 3
local DELETE_THRESHOLD_PERCENT = 0.005
local DELETE_THRESHOLD_MAX_SECONDS = 10
local INPUT_SECTION = "chapter-edit-mode"
local SUPPRESSOR_NAME = "chapter-edit-mode"

local function state_value(on)
    return on and "yes" or "no"
end

local function matches_toggle_binding(entry)
    return keybar_component.command_equals("script-binding chapter_edit_mode/toggle")(entry)
        or keybar_component.command_equals("script-binding chapter-edit-mode/toggle")(entry)
end

local function matches_script_binding(...)
    local names = { ... }
    return function(entry)
        for _, name in ipairs(names) do
            if keybar_component.command_equals("script-binding " .. name)(entry) then
                return true
            end
        end

        return false
    end
end

local control_items = {
    {
        id = "exit",
        section = "mode",
        fallback = "c",
        prefer = { "c" },
        desc = " EXIT",
        match = matches_toggle_binding,
    },
    {
        id = "add",
        section = "mode",
        fallback = "+",
        prefer = { "+", "KP_ADD" },
        desc = " ADD",
        match = matches_script_binding("chapter-edit-mode/add", "chapter_edit_mode/add"),
    },
    {
        id = "delete",
        section = "mode",
        fallback = "Meta+BS",
        prefer = { "Meta+BS", "Meta+DEL" },
        desc = " DELETE NEAREST",
        match = matches_script_binding("chapter-edit-mode/delete-nearest", "chapter_edit_mode/delete-nearest"),
    },
    {
        id = "delete_all",
        section = "mode",
        fallback = "Meta+Shift+BS",
        prefer = { "Meta+Shift+BS", "Meta+Shift+DEL" },
        desc = " DELETE ALL",
        match = matches_script_binding("chapter-edit-mode/delete-all", "chapter_edit_mode/delete-all"),
    },
}

local function broadcast_state()
    mp.commandv("script-message", "chapter-edit-mode-state", state_value(enabled))
end

local function set_keybar_suppressed(on)
    mp.commandv("script-message-to", "keybar", "keybar-suppress", SUPPRESSOR_NAME, state_value(on))
    broadcast_state()
end

local function refresh_resolved_keys()
    local input_path = mp.find_config_file("input.conf")
    local bindings = keybar_component.read_input_bindings(input_path)
    resolved_keys = keybar_component.resolve_item_keys(control_items, bindings)
end

local function format_time(seconds)
    seconds = math.max(tonumber(seconds) or 0, 0)
    local hours = math.floor(seconds / 3600)
    local minutes = math.floor((seconds % 3600) / 60)
    local secs = seconds % 60

    if hours > 0 then
        return ("%d:%02d:%05.2f"):format(hours, minutes, secs)
    end

    return ("%d:%05.2f"):format(minutes, secs)
end

local function chapter_list()
    local chapters = mp.get_property_native("chapter-list")
    if type(chapters) ~= "table" then
        return {}
    end

    local clean = {}
    for _, chapter in ipairs(chapters) do
        local time = tonumber(chapter.time)
        if time then
            table.insert(clean, {
                time = time,
                title = tostring(chapter.title or ""),
            })
        end
    end

    table.sort(clean, function(a, b)
        return a.time < b.time
    end)

    return clean
end

local function set_chapter_list(chapters)
    local ok, err = pcall(mp.set_property_native, "chapter-list", chapters)
    if not ok then
        mp.osd_message("Chapter edit: failed to update chapters", 3)
        mp.msg.error("failed to update chapter-list: " .. tostring(err))
        return false
    end

    return true
end

local function delete_threshold()
    local duration = mp.get_property_number("duration", 0) or 0
    local by_percent = duration * DELETE_THRESHOLD_PERCENT
    local threshold = math.max(DELETE_THRESHOLD_SECONDS, by_percent)
    return math.min(threshold, DELETE_THRESHOLD_MAX_SECONDS)
end

local function nearest_chapter(pos)
    local chapters = chapter_list()
    local best_index = nil
    local best_distance = nil

    for i, chapter in ipairs(chapters) do
        local distance = math.abs(chapter.time - pos)
        if not best_distance or distance < best_distance then
            best_index = i
            best_distance = distance
        end
    end

    return best_index, best_distance, chapters
end

local function render_progress(dim)
    if not enabled then
        progress_overlay:remove()
        return
    end

    dim = dim or mp.get_property_native("osd-dimensions")
    if not dim or not dim.w or not dim.h then
        return
    end

    local pos = mp.get_property_number("percent-pos")
    if not pos then
        return
    end

    local w, h = dim.w, dim.h
    local bar_h = 5
    local duration = mp.get_property_number("duration", 0) or 0
    local ass = assdraw.ass_new()

    ass:new_event()
    ass:append("{\\bord0\\shad0\\1c&H000000&}")
    ass:pos(0, h - bar_h)
    ass:draw_start()
    ass:rect_cw(0, 0, w, bar_h)
    ass:draw_stop()

    ass:new_event()
    ass:append("{\\bord0\\shad0\\1c&H9CFF00&}")
    ass:pos(0, h - bar_h)
    ass:draw_start()
    ass:rect_cw(0, 0, w * pos / 100, bar_h)
    ass:draw_stop()

    if duration > 0 then
        ass:new_event()
        ass:append("{\\bord0\\shad0\\1c&H00FFFF&}")
        ass:pos(0, h - bar_h)
        ass:draw_start()
        for _, chapter in ipairs(chapter_list()) do
            if chapter.time >= 0 and chapter.time <= duration then
                local x = math.floor(w * chapter.time / duration)
                ass:rect_cw(x - 1, -7, x + 1, bar_h)
            end
        end
        ass:draw_stop()
    end

    progress_overlay.data = ass.text
    progress_overlay.res_x = w
    progress_overlay.res_y = h
    progress_overlay:update()
end

local function render_bar()
    if not enabled then
        bar_overlay:remove()
        return
    end

    local dim = mp.get_property_native("osd-dimensions")
    if not dim or not dim.w or dim.w <= 0 or not dim.h or dim.h <= 0 then
        return
    end

    bar_overlay.data = keybar_component.build_bar(control_items, {
        dim = dim,
        title = "chapter edit mode",
        resolved_keys = resolved_keys,
        key_color = "{\\1c&H9CFF00&}",
        title_color = "{\\1c&H00FFFF&}",
    })
    bar_overlay.res_x = dim.w
    bar_overlay.res_y = dim.h
    bar_overlay:update()
end

local function render()
    local dim = mp.get_property_native("osd-dimensions")
    render_bar(dim)
    render_progress(dim)
end

local function add_chapter()
    if not enabled then
        mp.osd_message("Chapter edit mode is off", 1.5)
        return
    end

    local pos = mp.get_property_number("time-pos")
    if not pos then
        mp.osd_message("Chapter edit: no playback position", 2)
        return
    end

    local chapters = chapter_list()
    table.insert(chapters, {
        time = pos,
        title = "Chapter " .. format_time(pos),
    })
    table.sort(chapters, function(a, b)
        return a.time < b.time
    end)

    if set_chapter_list(chapters) then
        mp.osd_message("Chapter added @ " .. format_time(pos), 1.5)
        render()
    end
end

local function delete_nearest_chapter()
    if not enabled then
        mp.osd_message("Chapter edit mode is off", 1.5)
        return
    end

    local pos = mp.get_property_number("time-pos")
    if not pos then
        mp.osd_message("Chapter edit: no playback position", 2)
        return
    end

    local index, distance, chapters = nearest_chapter(pos)
    if not index then
        mp.osd_message("Chapter edit: no chapters", 2)
        return
    end

    local threshold = delete_threshold()
    if distance > threshold then
        mp.osd_message(("No chapter within %.1fs"):format(threshold), 2)
        return
    end

    local removed = table.remove(chapters, index)
    if set_chapter_list(chapters) then
        mp.osd_message("Deleted chapter @ " .. format_time(removed.time), 1.5)
        render()
    end
end

local function delete_all_chapters()
    if not enabled then
        mp.osd_message("Chapter edit mode is off", 1.5)
        return
    end

    local chapters = chapter_list()
    if #chapters == 0 then
        mp.osd_message("Chapter edit: no chapters", 2)
        return
    end

    if set_chapter_list({}) then
        mp.osd_message(("Deleted %d chapters"):format(#chapters), 1.5)
        render()
    end
end

local set_enabled

local function enable_bindings()
    mp.commandv("enable-section", INPUT_SECTION)
end

local function disable_bindings()
    mp.commandv("disable-section", INPUT_SECTION)
end

function set_enabled(next_enabled)
    next_enabled = not not next_enabled
    if enabled == next_enabled then
        return
    end

    enabled = next_enabled
    if enabled then
        refresh_resolved_keys()
        enable_bindings()
        set_keybar_suppressed(true)
        mp.osd_message("chapter edit mode", 1.5)
        render()
    else
        disable_bindings()
        bar_overlay:remove()
        progress_overlay:remove()
        set_keybar_suppressed(false)
        mp.osd_message("chapter edit mode off", 1.5)
    end
end

local function toggle()
    set_enabled(not enabled)
end

mp.observe_property("osd-dimensions", "native", function()
    render()
end)

mp.observe_property("percent-pos", "number", function()
    render_progress()
end)

mp.observe_property("chapter-list", "native", function()
    render()
end)

mp.register_event("file-loaded", function()
    if enabled then
        refresh_resolved_keys()
        render()
    end
end)

mp.register_script_message("toggle", toggle)
mp.register_script_message("enable", function() set_enabled(true) end)
mp.register_script_message("disable", function() set_enabled(false) end)
mp.register_script_message("add", add_chapter)
mp.register_script_message("delete-nearest", delete_nearest_chapter)
mp.register_script_message("delete-all", delete_all_chapters)
mp.register_script_message("query", broadcast_state)
mp.register_script_message("chapter-edit-mode-query", broadcast_state)

mp.add_key_binding(nil, "toggle", toggle)
mp.add_key_binding(nil, "add", add_chapter)
mp.add_key_binding(nil, "delete-nearest", delete_nearest_chapter)
mp.add_key_binding(nil, "delete-all", delete_all_chapters)
mp.add_key_binding(nil, "exit", function() set_enabled(false) end)
mp.register_event("shutdown", function()
    if enabled then
        disable_bindings()
        set_keybar_suppressed(false)
    end
end)
mp.add_timeout(0, broadcast_state)
