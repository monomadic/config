local mp = require "mp"

local popup_visible = false
local SHOW_DURATION_MS = 600000
local WRAP_WIDTH = 110

local TRACK_TAG_FIELDS = {
    "title",
    "lang",
    "langs",
    "default",
    "forced",
    "selected",
    "external",
    "main-selection",
    "dependent",
    "visual-impaired",
    "hearing-impaired",
    "image",
    "albumart",
}

local function is_blank(value)
    return value == nil or value == ""
end

local function scalar_to_string(value)
    local value_type = type(value)
    if value_type == "boolean" then
        return value and "yes" or "no"
    end
    if value_type == "number" then
        if value == math.floor(value) then
            return tostring(math.floor(value))
        end
        local formatted = string.format("%.3f", value)
        formatted = formatted:gsub("0+$", ""):gsub("%.$", "")
        return formatted
    end
    return tostring(value)
end

local function has_content(value)
    if value == nil then
        return false
    end

    if type(value) ~= "table" then
        return scalar_to_string(value) ~= ""
    end

    for _, nested in pairs(value) do
        if has_content(nested) then
            return true
        end
    end

    return false
end

local function sorted_keys(tbl)
    local keys = {}
    for key in pairs(tbl or {}) do
        table.insert(keys, key)
    end

    table.sort(keys, function(a, b)
        local ta, tb = type(a), type(b)
        if ta == tb then
            return tostring(a):lower() < tostring(b):lower()
        end
        return ta < tb
    end)

    return keys
end

local function is_array(tbl)
    if type(tbl) ~= "table" then
        return false
    end

    local count = 0
    for key in pairs(tbl) do
        if type(key) ~= "number" or key < 1 or key ~= math.floor(key) then
            return false
        end
        count = count + 1
    end

    for index = 1, count do
        if tbl[index] == nil then
            return false
        end
    end

    return true
end

local function wrap_line(lines, prefix, value)
    local prefix_len = #prefix

    for raw_line in tostring(value):gmatch("([^\n]*)\n?") do
        if raw_line == "" then
            table.insert(lines, prefix)
        else
            local text = prefix .. raw_line
            while #text > WRAP_WIDTH do
                local split_at
                local min_split = math.max(prefix_len + 1, math.floor(WRAP_WIDTH * 0.55))
                for i = WRAP_WIDTH, min_split, -1 do
                    local ch = text:sub(i, i)
                    if ch == " " or ch == "/" or ch == "&" or ch == "?" or ch == "," or ch == ";" then
                        split_at = i
                        break
                    end
                end

                split_at = split_at or WRAP_WIDTH
                table.insert(lines, text:sub(1, split_at))
                text = string.rep(" ", prefix_len) .. text:sub(split_at + 1):gsub("^%s+", "")
            end
            table.insert(lines, text)
        end
    end
end

local function append_value(lines, label, value, indent)
    indent = indent or ""

    if value == nil then
        return
    end

    if type(value) ~= "table" then
        local text = scalar_to_string(value)
        if text ~= "" then
            wrap_line(lines, indent .. label .. ": ", text)
        end
        return
    end

    if not has_content(value) then
        return
    end

    table.insert(lines, indent .. label .. ":")

    if is_array(value) then
        for index, item in ipairs(value) do
            local item_label = "[" .. index .. "]"
            if type(item) == "table" then
                if has_content(item) then
                    table.insert(lines, indent .. "  " .. item_label)
                    for _, key in ipairs(sorted_keys(item)) do
                        append_value(lines, tostring(key), item[key], indent .. "    ")
                    end
                end
            else
                append_value(lines, item_label, item, indent .. "  ")
            end
        end
        return
    end

    for _, key in ipairs(sorted_keys(value)) do
        append_value(lines, tostring(key), value[key], indent .. "  ")
    end
end

local function append_section_contents(lines, value)
    if type(value) ~= "table" or not has_content(value) then
        return
    end

    if is_array(value) then
        for index, item in ipairs(value) do
            append_value(lines, "[" .. index .. "]", item, "")
        end
        return
    end

    for _, key in ipairs(sorted_keys(value)) do
        append_value(lines, tostring(key), value[key], "")
    end
end

local function collect_source_fields()
    local fields = {}
    local seen = {}

    local ordered = {
        { label = "Media title", value = mp.get_property("media-title") },
        { label = "Open URL", value = mp.get_property("stream-open-filename") },
        { label = "Stream path", value = mp.get_property("stream-path") },
        { label = "Path", value = mp.get_property("path") },
        { label = "Playlist path", value = mp.get_property("playlist-path") },
        { label = "Filename", value = mp.get_property("filename") },
    }

    for _, field in ipairs(ordered) do
        local value = field.value
        if not is_blank(value) and not seen[value] then
            seen[value] = true
            table.insert(fields, {
                label = field.label,
                value = value,
            })
        end
    end

    return fields
end

local function collect_track_tags()
    local tracks = mp.get_property_native("track-list") or {}
    local out = {}

    for _, track in ipairs(tracks) do
        local tags = {}

        if not is_blank(track.type) then
            tags.type = track.type
        end
        if track.id ~= nil then
            tags.id = track.id
        end

        for _, field in ipairs(TRACK_TAG_FIELDS) do
            local value = track[field]
            if type(value) == "table" then
                if has_content(value) then
                    tags[field] = value
                end
            elseif not is_blank(value) then
                tags[field] = value
            end
        end

        if has_content(tags) then
            table.insert(out, tags)
        end
    end

    return out
end

local function build_popup_text()
    local lines = { "Metadata + Tags" }

    local function add_section(title, value, custom)
        if not has_content(value) then
            return
        end

        table.insert(lines, "")
        table.insert(lines, title)
        if custom then
            custom(value)
        else
            append_section_contents(lines, value)
        end
    end

    local source_fields = collect_source_fields()
    if #source_fields > 0 then
        table.insert(lines, "")
        table.insert(lines, "Source")
        for _, field in ipairs(source_fields) do
            append_value(lines, field.label, field.value, "")
        end
    end

    add_section("File metadata", mp.get_property_native("metadata"))
    add_section("Chapter metadata", mp.get_property_native("chapter-metadata"))
    add_section("Filtered metadata", mp.get_property_native("filtered-metadata"))
    add_section("Video filter metadata", mp.get_property_native("vf-metadata"))
    add_section("Audio filter metadata", mp.get_property_native("af-metadata"))
    add_section("Track tags", collect_track_tags())

    if #lines == 1 then
        table.insert(lines, "")
        table.insert(lines, "No metadata or tags found for this file.")
    end

    return table.concat(lines, "\n")
end

local function render_popup()
    mp.commandv("show-text", build_popup_text(), SHOW_DURATION_MS, 1)
end

local function clear_popup()
    mp.commandv("show-text", "", 0, 1)
end

local function toggle_popup()
    popup_visible = not popup_visible
    if popup_visible then
        render_popup()
    else
        clear_popup()
    end
end

mp.add_key_binding(nil, "toggle_full_metadata_popup", toggle_popup)
mp.register_script_message("toggle_full_metadata_popup", toggle_popup)

mp.register_event("file-loaded", function()
    if popup_visible then
        render_popup()
    end
end)

mp.register_event("end-file", function()
    popup_visible = false
end)
