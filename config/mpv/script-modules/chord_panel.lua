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

local M = {}

local Panel = {}
Panel.__index = Panel

function M.new(spec)
    local panel = setmetatable({
        name = assert(spec.name, "chord_panel needs a name"),
        title = spec.title or spec.name,
        hint = spec.hint or "press a key",
        timeout = tonumber(spec.timeout) or 10,
        actions = spec.actions or {},
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

    local w, h = dim.w, dim.h
    local panel_w = math.min(560, math.floor(w * 0.76))
    local row_h = 30
    local panel_h = 64 + (#self.actions * row_h) + 20
    local x = math.floor((w - panel_w) / 2)
    local y = math.floor(math.max(40, h * 0.15))
    local ass = assdraw.ass_new()

    ass:new_event()
    ass:append(("{\\an7\\pos(%d,%d)\\bord0\\shad0\\1c&H111111&\\alpha&H20&}"):format(x, y))
    ass:draw_start()
    ass:rect_cw(0, 0, panel_w, panel_h)
    ass:draw_stop()

    ass:new_event()
    ass:append(("{\\an7\\pos(%d,%d)\\bord0\\shad0\\fs24\\1c&H00FFFF&}%s\\N"):format(x + 24, y + 18, self.title))
    ass:append(("{\\fs16\\1c&HAAAAAA&}%s"):format(self.hint))

    for i, item in ipairs(self.actions) do
        local row_y = y + 58 + ((i - 1) * row_h)
        ass:new_event()
        ass:append(("{\\an7\\pos(%d,%d)\\bord0\\shad0\\fs20\\1c&H9CFF00&}%s"):format(
            x + 26, row_y, item.key_label or item.key))
        ass:append(("{\\1c&HFFFFFF&}  %s"):format(item.label))
    end

    ass:new_event()
    ass:append(("{\\an7\\pos(%d,%d)\\bord0\\shad0\\fs16\\1c&H777777&}ESC cancel"):format(x + 26, y + panel_h - 28))

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
