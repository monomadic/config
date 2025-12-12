-- ~/.config/mpv/scripts/keybar.lua

local function render_bar()
    local w, h = mp.get_osd_size()
    if not w or w <= 0 or not h or h <= 0 then
        return
    end
    -- bottom-center, size 20
    local ass = "{\\an2}{\\fs20}KEYBAR TEST"
    mp.set_osd_ass(w, h, ass)
end

mp.register_event("file-loaded", render_bar)
