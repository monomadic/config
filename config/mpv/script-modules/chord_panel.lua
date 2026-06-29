-- chord_panel.lua - reusable keyboard-chord panel.
--
-- Shows a small overlay listing single-key actions, grabs those keys (via
-- forced bindings, so they override input.conf) while visible, runs the
-- picked action, then dismisses. ESC or a timeout cancels.
--
-- Usage:
--   local chord_panel = <loadfile script-modules/chord_panel.lua>
--   local panel = chord_panel.new({
--       name = "my-panel",        -- unique, used for binding names
--       title = "do thing",
--       timeout = 10,             -- seconds before auto-dismiss
--       actions = {
--           { key = "x", label = "do x", fn = function() ... end },
--           -- several keys can share one row; fn receives the pressed key
--           { keys = { "1", "2" }, key_label = "1-2", label = "...", fn = function(key) ... end },
--       },
--   })
--   panel:show() / panel:hide() / panel:toggle()
local mp = require "mp"
local assdraw = require "mp.assdraw"

-- ── Panel appearance ────────────────────────────────────────────────────────
-- Everything visual lives here so it can be tuned in one place. Font sizes are
-- a fraction of the OSD height, then clamped to a pixel range, so the panel
-- stays legible across window sizes and hi-dpi displays (fixed pixel sizes
-- looked tiny on retina, where osd-dimensions reports backing-store pixels).
-- Bump base_scale (or base_min/base_max) to enlarge the whole panel at once.
-- Individual panels can override any of these via `theme = { ... }` in new().
local THEME = {
    font = nil, -- panel font; nil keeps mpv's default OSD font

    -- Base text size, then per-role multipliers relative to it.
    base_scale = 0.024, base_min = 18, base_max = 36,
    title_mult = 1.30,  -- panel heading
    hint_mult  = 0.80,  -- "press a key" line
    row_mult   = 1.05,  -- action rows (key + label)
    foot_mult  = 0.80,  -- "ESC cancel" footer

    -- Colours (ASS BGR hex strings) and the panel background.
    bg_color    = "&H111111&",
    bg_alpha    = "&H20&",
    title_color = "&H00FFFF&",
    hint_color  = "&HAAAAAA&",
    key_color   = "&H9CFF00&",
    label_color = "&HFFFFFF&",
    foot_color  = "&H777777&",

    -- Layout.
    width_frac = 0.76, -- panel width as a fraction of OSD width …
    width_max  = 640,  -- … but never wider than this many pixels
    pad_x      = 24,   -- horizontal text inset
}
-- ────────────────────────────────────────────────────────────────────────────

local function clamp(value, lo, hi)
    return math.max(lo, math.min(hi, value))
end

local M = {}

local Panel = {}
Panel.__index = Panel

function M.new(spec)
    -- Start from the module defaults, then layer any per-panel overrides on top.
    local theme = {}
    for key, value in pairs(THEME) do
        theme[key] = value
    end
    if spec.theme then
        for key, value in pairs(spec.theme) do
            theme[key] = value
        end
    end

    local panel = setmetatable({
        name = assert(spec.name, "chord_panel needs a name"),
        title = spec.title or spec.name,
        hint = spec.hint or "press a key",
        timeout = tonumber(spec.timeout) or 10,
        actions = spec.actions or {},
        theme = theme,
        overlay = mp.create_osd_overlay("ass-events"),
        active = false,
        hide_timer = nil,
        grabbed = {},
    }, Panel)
    panel.overlay.z = spec.z or 220

    mp.observe_property("osd-dimensions", "native", function()
        if panel.active then
            panel:render()
        end
    end)
    mp.register_event("shutdown", function()
        panel:stop_timer()
    end)

    return panel
end

function Panel:stop_timer()
    if self.hide_timer then
        self.hide_timer:kill()
        self.hide_timer = nil
    end
end

function Panel:grab_keys()
    local function grab(key, fn)
        local binding = ("%s-key-%d"):format(self.name, #self.grabbed + 1)
        mp.add_forced_key_binding(key, binding, fn)
        table.insert(self.grabbed, binding)
    end

    for _, action in ipairs(self.actions) do
        for _, key in ipairs(action.keys or { action.key }) do
            grab(key, function()
                self:hide()
                action.fn(key)
            end)
        end
    end

    grab("ESC", function()
        self:hide()
    end)
end

function Panel:release_keys()
    for _, binding in ipairs(self.grabbed) do
        mp.remove_key_binding(binding)
    end
    self.grabbed = {}
end

function Panel:render()
    local dim = mp.get_property_native("osd-dimensions")
    if not dim or not dim.w or dim.w <= 0 or not dim.h or dim.h <= 0 then
        local lines = { self.title }
        for _, item in ipairs(self.actions) do
            table.insert(lines, (item.key_label or item.key) .. " " .. item.label)
        end
        table.insert(lines, "ESC cancel")
        mp.osd_message(table.concat(lines, "\n"), self.timeout)
        return
    end

    local t = self.theme
    local w, h = dim.w, dim.h

    -- Resolution-aware font sizes, all derived from one base size.
    local base = math.floor(clamp(h * t.base_scale, t.base_min, t.base_max))
    local fs_title = math.floor(base * t.title_mult)
    local fs_hint = math.floor(base * t.hint_mult)
    local fs_row = math.floor(base * t.row_mult)
    local fs_foot = math.floor(base * t.foot_mult)
    local font = t.font and ("\\fn" .. t.font) or ""

    -- Vertical layout, measured from the panel top so it scales with the fonts.
    local pad_x = t.pad_x
    local pad_y = math.floor(base * 0.85)
    local row_h = math.floor(fs_row * 1.55)
    local title_off = pad_y
    local hint_off = title_off + math.floor(fs_title * 1.15)
    local rows_off = hint_off + math.floor(fs_hint * 1.70)
    local foot_off = rows_off + (#self.actions * row_h) + math.floor(fs_foot * 0.5)
    local panel_h = foot_off + math.floor(fs_foot * 1.2) + pad_y

    local panel_w = math.min(t.width_max, math.floor(w * t.width_frac))
    local x = math.floor((w - panel_w) / 2)
    local y = math.floor(math.max(40, h * 0.15))
    local ass = assdraw.ass_new()

    ass:new_event()
    ass:append(("{\\an7\\pos(%d,%d)\\bord0\\shad0\\1c%s\\alpha%s}"):format(x, y, t.bg_color, t.bg_alpha))
    ass:draw_start()
    ass:rect_cw(0, 0, panel_w, panel_h)
    ass:draw_stop()

    ass:new_event()
    ass:append(("{\\an7\\pos(%d,%d)\\bord0\\shad0%s\\fs%d\\1c%s}%s"):format(
        x + pad_x, y + title_off, font, fs_title, t.title_color, self.title))

    ass:new_event()
    ass:append(("{\\an7\\pos(%d,%d)\\bord0\\shad0%s\\fs%d\\1c%s}%s"):format(
        x + pad_x, y + hint_off, font, fs_hint, t.hint_color, self.hint))

    for i, item in ipairs(self.actions) do
        local row_y = y + rows_off + ((i - 1) * row_h)
        ass:new_event()
        ass:append(("{\\an7\\pos(%d,%d)\\bord0\\shad0%s\\fs%d\\1c%s}%s"):format(
            x + pad_x + 2, row_y, font, fs_row, t.key_color, item.key_label or item.key))
        ass:append(("{\\1c%s}  %s"):format(t.label_color, item.label))
    end

    ass:new_event()
    ass:append(("{\\an7\\pos(%d,%d)\\bord0\\shad0%s\\fs%d\\1c%s}ESC cancel"):format(
        x + pad_x + 2, y + foot_off, font, fs_foot, t.foot_color))

    self.overlay.data = ass.text
    self.overlay.res_x = w
    self.overlay.res_y = h
    self.overlay:update()
end

function Panel:show()
    if not self.active then
        self.active = true
        self:grab_keys()
    end
    self:render()
    self:stop_timer()
    self.hide_timer = mp.add_timeout(self.timeout, function()
        self:hide()
    end)
end

function Panel:hide()
    self:stop_timer()
    self.overlay:remove()
    if self.active then
        self:release_keys()
        self.active = false
    end
end

function Panel:toggle()
    if self.active then
        self:hide()
    else
        self:show()
    end
end

return M
