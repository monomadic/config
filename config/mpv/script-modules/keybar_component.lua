local M = {}

local function trim(s)
    return (s or ""):match("^%s*(.-)%s*$")
end

M.trim = trim

function M.normalize_command(command)
    command = trim(command)
    command = command:gsub("^script_binding%s+", "script-binding ")
    command = command:gsub("%s+", " ")
    return command
end

function M.normalize_binding_key(binding)
    if tostring(binding or "") == "+" then
        return "+"
    end

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
            bs = "BS",
            del = "DEL",
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

function M.format_key_label(binding)
    if tostring(binding or "") == "+" then
        return "+"
    end

    local mods = {
        Meta = "⌘",
        Ctrl = "⌃",
        Alt = "⌥",
        Shift = "⇧",
    }
    local keys = {
        BS = "⌫",
        DEL = "⌦",
        ESC = "ESC",
        TAB = "TAB",
        SPACE = "SPACE",
        ENTER = "ENTER",
        LEFT = "←",
        RIGHT = "→",
    }
    local parts = {}
    local out = {}

    binding = M.normalize_binding_key(binding)

    for part in binding:gmatch("[^+]+") do
        table.insert(parts, part)
    end

    for _, part in ipairs(parts) do
        table.insert(out, mods[part] or keys[part] or part)
    end

    return table.concat(out)
end

function M.command_equals(target)
    target = M.normalize_command(target)
    return function(entry)
        return entry.command == target
    end
end

function M.command_prefix(prefix)
    prefix = M.normalize_command(prefix)
    return function(entry)
        return entry.command:sub(1, #prefix) == prefix
    end
end

function M.command_contains(...)
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

function M.read_input_bindings(path)
    local bindings = {}
    local file = path and io.open(path, "r") or nil

    if not file then
        return bindings
    end

    for line in file:lines() do
        local cleaned = trim(line)
        if cleaned ~= "" and not cleaned:match("^#") then
            local key, command = cleaned:match("^(%S+)%s+(.+)$")
            if key and command then
                command = command:gsub("^%b{}%s+", "")
                table.insert(bindings, {
                    key = M.normalize_binding_key(key),
                    command = M.normalize_command(command),
                })
            end
        end
    end

    file:close()
    return bindings
end

function M.resolve_item_key(item, bindings)
    local matches = {}

    for _, entry in ipairs(bindings or {}) do
        if item.match and item.match(entry) then
            table.insert(matches, entry.key)
        end
    end

    if item.prefer then
        for _, preferred in ipairs(item.prefer) do
            preferred = M.normalize_binding_key(preferred)
            for _, candidate in ipairs(matches) do
                if candidate == preferred then
                    return M.format_key_label(candidate)
                end
            end
        end
    end

    if #matches > 0 then
        return M.format_key_label(matches[1])
    end

    return M.format_key_label(item.fallback)
end

function M.resolve_item_keys(items, bindings)
    local resolved = {}

    for _, item in ipairs(items or {}) do
        if item.id then
            resolved[item.id] = M.resolve_item_key(item, bindings)
        end
    end

    return resolved
end

local function badge(on, text_color)
    local c = on and "{\\1c&H00FFFF&}" or "{\\1c&H777777&}"
    return c .. (on and "ON" or "OFF") .. text_color
end

local function render_item(item, key_label, colors, context)
    local desc = item.desc
    if type(desc) == "function" then
        desc = desc(function(on) return badge(on, colors.text) end, context.fmt_delay or function() return "" end, context)
    end

    return colors.chip
        .. "  "
        .. colors.key
        .. trim(key_label)
        .. colors.text
        .. " "
        .. trim(desc)
        .. "  "
end

function M.build_bar(items, opts)
    opts = opts or {}
    local dim = opts.dim or {}
    local w = dim.w or 1280
    local h = dim.h or 720
    local scale = opts.scale or 9
    local progress_h = opts.progress_h or 5
    local bar_h = opts.bar_h or 26
    local text_y = h - progress_h
    local y0 = h - progress_h - bar_h
    local colors = {
        key  = opts.key_color or "{\\1c&H9CFF00&}",
        text = opts.text_color or "{\\1c&HFFFFFF&}",
        chip = opts.chip_color or "{\\1c&H181818&\\alpha&H00&}",
        sep  = opts.sep_color or "{\\1c&H6A6A6A&}",
        title = opts.title_color or "{\\1c&H00FFFF&}",
    }
    local bg_alpha = opts.bg_alpha or "&HA0&"
    local bg = ("{\\an7\\pos(0,%d)\\bord0\\shad0\\1c&H000000&\\alpha%s\\p1}"
        .. "m 0 0 l %d 0 l %d %d l 0 %d"
        .. "{\\p0}"):format(y0, bg_alpha, w, w, bar_h, bar_h)
    local separator = colors.sep .. "|" .. colors.text
    local s = ("{\\an2\\pos(%d,%d)\\bord0\\shad0\\scale%d}"):format(
        math.floor(w / 2),
        text_y,
        scale
    )

    local groups = {}
    if opts.title and opts.title ~= "" then
        table.insert(groups, colors.title .. trim(opts.title) .. colors.text)
    end

    local current_group = nil
    local last_section = nil
    local resolved_keys = opts.resolved_keys or {}
    local context = opts.context or {}

    for _, item in ipairs(items or {}) do
        if item.section ~= last_section then
            current_group = {}
            table.insert(groups, current_group)
            last_section = item.section
        end

        local label = item.key or resolved_keys[item.id] or M.format_key_label(item.fallback)
        table.insert(current_group, render_item(item, label, colors, context))
    end

    local rendered_groups = {}
    for _, group in ipairs(groups) do
        if type(group) == "table" then
            table.insert(rendered_groups, table.concat(group, separator))
        else
            table.insert(rendered_groups, group)
        end
    end

    return bg .. "\n" .. s .. table.concat(rendered_groups, separator)
end

return M
