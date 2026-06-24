-- info-panel.lua - toggleable file/metadata panel in the top-right corner.
-- Bound to "i" in input.conf (script-message toggle-info-panel).
local mp = require "mp"

local overlay = mp.create_osd_overlay("ass-events")
overlay.z = 30

local visible = false

local PANEL_FONT = "Helvetica Neue"
local WRAP_CHARS = 54
local MAX_VALUE_LINES = 6 -- per field, keeps long descriptions in check
local BG_ALPHA = "&H70&"

-- Metadata keys already surfaced by the curated fields above; these are
-- excluded from the "extra tags" dump so nothing shows twice. Keys are stored
-- in the same normalized form metadata_lookup uses (lowercase, alphanumerics).
local CONSUMED_KEYS = {
    title = true,
    description = true, comment = true, synopsis = true, plot = true, summary = true,
    actors = true, actor = true, cast = true, starring = true, performer = true,
}

local TITLE_COLOR = "{\\1c&HF8F8F8&}"
local DESC_COLOR = "{\\1c&HC8C8C8&}"
local LABEL_COLOR = "{\\1c&H00FFFF&}"
local VALUE_COLOR = "{\\1c&HF2F2F2&}"

local function trim(text)
    return tostring(text or ""):match("^%s*(.-)%s*$")
end

local function ass_escape(text)
    text = text:gsub("\\", "\\\239\187\191")
    text = text:gsub("{", "\\{")
    text = text:gsub("}", "\\}")
    return text
end

local function rounded_rect_path(x0, y0, x1, y1, r)
    local c = r * 0.55228475

    return table.concat({
        string.format("m %.1f %.1f", x0 + r, y0),
        string.format("l %.1f %.1f", x1 - r, y0),
        string.format("b %.1f %.1f %.1f %.1f %.1f %.1f", x1 - r + c, y0, x1, y0 + r - c, x1, y0 + r),
        string.format("l %.1f %.1f", x1, y1 - r),
        string.format("b %.1f %.1f %.1f %.1f %.1f %.1f", x1, y1 - r + c, x1 - r + c, y1, x1 - r, y1),
        string.format("l %.1f %.1f", x0 + r, y1),
        string.format("b %.1f %.1f %.1f %.1f %.1f %.1f", x0 + r - c, y1, x0, y1 - r + c, x0, y1 - r),
        string.format("l %.1f %.1f", x0, y0 + r),
        string.format("b %.1f %.1f %.1f %.1f %.1f %.1f", x0, y0 + r - c, x0 + r - c, y0, x0 + r, y0),
    }, " ")
end

local function metadata_value_to_text(value)
    if type(value) == "table" then
        local parts = {}

        if #value > 0 then
            for _, item in ipairs(value) do
                local text = trim(item)
                if text ~= "" then
                    table.insert(parts, text)
                end
            end
        else
            for _, item in pairs(value) do
                local text = trim(item)
                if text ~= "" then
                    table.insert(parts, text)
                end
            end
            table.sort(parts)
        end

        return table.concat(parts, ", ")
    end

    return trim(value)
end

local function metadata_lookup(preferred_keys)
    local sources = {
        mp.get_property_native("metadata") or {},
        mp.get_property_native("filtered-metadata") or {},
    }

    for _, source in ipairs(sources) do
        local normalized = {}

        for key, value in pairs(source) do
            normalized[tostring(key):lower():gsub("[^%w]", "")] = value
        end

        for _, key in ipairs(preferred_keys) do
            local text = metadata_value_to_text(normalized[key])
            if text ~= "" then
                return text
            end
        end
    end

    return nil
end

local function is_url(text)
    return text and text:match("^%a[%w+.-]*://") ~= nil
end

local function get_url()
    local path = mp.get_property("path") or ""
    if is_url(path) then
        return path
    end

    local open = mp.get_property("stream-open-filename") or ""
    if open ~= path and is_url(open) then
        return open
    end

    return nil
end

local function get_location()
    local path = mp.get_property("path")
    if not path or path == "" or is_url(path) then
        return nil
    end

    local dir = path:match("^(.*)/[^/]+$")
    if not dir or dir == "" then
        return mp.get_property("working-directory")
    end

    if not dir:match("^/") then
        local cwd = mp.get_property("working-directory")
        if cwd and cwd ~= "" then
            dir = cwd .. "/" .. dir
        end
    end

    return dir
end

-- Any container metadata tags not already shown by the curated fields, sorted
-- alphabetically so ordering is stable between files.
local function collect_extra_tags()
    local meta = mp.get_property_native("metadata") or {}
    local seen = {}
    local extras = {}

    for key, value in pairs(meta) do
        local norm = tostring(key):lower():gsub("[^%w]", "")
        if not CONSUMED_KEYS[norm] and not seen[norm] then
            local text = metadata_value_to_text(value)
            if text ~= "" then
                seen[norm] = true
                table.insert(extras, { label = trim(key), value = text })
            end
        end
    end

    table.sort(extras, function(a, b) return a.label:lower() < b.label:lower() end)

    return extras
end

local function collect_fields()
    local fields = {}

    local function add(label, value)
        value = trim(value)
        if value ~= "" then
            table.insert(fields, { label = label, value = value })
        end
    end

    add("URL", get_url())
    add("Actors", metadata_lookup({ "actors", "actor", "cast", "starring", "performer" }))
    add("Filename", mp.get_property("filename"))
    add("Location", get_location())

    for _, extra in ipairs(collect_extra_tags()) do
        add(extra.label, extra.value)
    end

    return fields
end

local function wrap_value(value, width)
    local out = {}

    for paragraph in tostring(value):gmatch("[^\n]+") do
        local text = trim(paragraph)
        while #text > width do
            local split_at
            for i = width, math.floor(width * 0.5), -1 do
                local ch = text:sub(i, i)
                if ch == " " or ch == "/" or ch == "," or ch == ";" or ch == "&" or ch == "?" then
                    split_at = i
                    break
                end
            end
            split_at = split_at or width
            table.insert(out, trim(text:sub(1, split_at)))
            text = trim(text:sub(split_at + 1))
        end
        if text ~= "" then
            table.insert(out, text)
        end
        if #out >= MAX_VALUE_LINES then
            out[MAX_VALUE_LINES] = out[MAX_VALUE_LINES] .. " …"
            break
        end
    end

    return out
end

local function render()
    if not visible then
        overlay:remove()
        return
    end

    local dim = mp.get_property_native("osd-dimensions")
    if not dim or not dim.w or dim.w <= 0 or not dim.h or dim.h <= 0 then
        return
    end
    local w, h = dim.w, dim.h

    local fs = math.floor(math.max(14, math.min(26, h * 0.017)))
    local fs_title = math.floor(fs * 1.30)
    local fs_desc = math.floor(fs * 0.92)
    local fs_label = math.floor(fs * 0.70)

    -- Build styled lines: title heading, description under it, then one
    -- LABEL-on-its-own-line block per field.
    local lines = {}
    local function push(text, size, color, bold, gap_before)
        table.insert(lines, {
            text = ass_escape(text),
            fs = size,
            color = color,
            bold = bold and 1 or 0,
            gap = gap_before or 0,
        })
    end

    local title = trim(metadata_lookup({ "title" }) or mp.get_property("media-title") or "")
    local desc = metadata_lookup({ "description", "comment", "synopsis", "plot", "summary" })

    if title ~= "" then
        for _, line in ipairs(wrap_value(title, math.floor(WRAP_CHARS / 1.3))) do
            push(line, fs_title, TITLE_COLOR, true)
        end
    end

    if desc then
        for index, line in ipairs(wrap_value(desc, WRAP_CHARS)) do
            push(line, fs_desc, DESC_COLOR, false, index == 1 and math.floor(fs * 0.3) or 0)
        end
    end

    for _, field in ipairs(collect_fields()) do
        push(field.label:upper(), fs_label, LABEL_COLOR, true, math.floor(fs * 0.8))
        for _, line in ipairs(wrap_value(field.value, WRAP_CHARS)) do
            push(line, fs, VALUE_COLOR, false)
        end
    end

    if #lines == 0 then
        push("No metadata available", fs, VALUE_COLOR, false)
    end

    -- Layout: variable line heights, estimated proportional-font width.
    local pad = math.floor(fs * 0.9)
    local margin = math.floor(math.max(10, h * 0.026))
    local max_px = 0
    local cursor = 0

    for _, line in ipairs(lines) do
        cursor = cursor + line.gap
        line.y_off = cursor
        cursor = cursor + math.floor(line.fs * 1.35)
        max_px = math.max(max_px, math.floor(#line.text * line.fs * 0.52))
    end

    local panel_w = math.min(math.floor(w * 0.42), max_px + pad * 2)
    local panel_h = cursor + pad * 2
    local x1 = w - margin
    local x0 = x1 - panel_w
    -- Sit below the playlist "N of M" pill that metadata.lua puts in the corner.
    local y0 = margin + math.floor(fs * 2.2)

    local events = {}
    table.insert(events, "{\\an7\\pos(0,0)\\bord0\\shad0\\1c&H000000&\\alpha" .. BG_ALPHA .. "\\p1}"
        .. rounded_rect_path(x0, y0, x1, y0 + panel_h, math.floor(fs * 0.5)) .. "{\\p0}")

    for _, line in ipairs(lines) do
        table.insert(events, string.format(
            "{\\an7\\pos(%d,%d)\\fn%s\\fs%d\\b%d\\bord0\\shad0}%s%s",
            x0 + pad, y0 + pad + line.y_off, PANEL_FONT, line.fs, line.bold, line.color, line.text
        ))
    end

    overlay.data = table.concat(events, "\n")
    overlay.res_x = w
    overlay.res_y = h
    overlay:update()
end

local function toggle()
    visible = not visible
    render()
end

mp.add_key_binding(nil, "toggle-info-panel", toggle)
mp.register_script_message("toggle-info-panel", toggle)

mp.register_event("file-loaded", function()
    if visible then
        render()
    end
end)

mp.observe_property("osd-dimensions", "native", function()
    if visible then
        render()
    end
end)

mp.observe_property("metadata", "native", function()
    if visible then
        render()
    end
end)
