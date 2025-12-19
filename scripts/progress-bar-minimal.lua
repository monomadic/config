local mp = require 'mp'
local assdraw = require 'mp.assdraw'

local function draw_bar()
    local pos = mp.get_property_number("percent-pos", 0)
    local w, h = mp.get_property_number("osd-width"), mp.get_property_number("osd-height")
    local bar_h = 3  -- bar height in pixels
    
    local ass = assdraw.ass_new()
    
    -- Black background
    ass:new_event()
    ass:append('{\\1c&H000000&}')  -- black
    ass:pos(0, h - bar_h)
    ass:draw_start()
    ass:rect_cw(0, 0, w, bar_h)
    ass:draw_stop()
    
    -- White progress
    ass:new_event()
    ass:append('{\\1c&HFFFFFF&}')  -- white
    ass:pos(0, h - bar_h)
    ass:draw_start()
    ass:rect_cw(0, 0, w * pos / 100, bar_h)
    ass:draw_stop()
    
    mp.set_osd_ass(w, h, ass.text)
end

mp.register_event("playback-restart", draw_bar)
mp.observe_property("percent-pos", "number", draw_bar)
