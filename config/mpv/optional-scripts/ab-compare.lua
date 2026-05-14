-- Opt-in A/B comparison controls.
--
-- Usage:
--   mpv --profile=ab /path/to/A.mp4 /path/to/B.mp4
-- or:
--   mpv --profile=ab --external-file=/path/to/B.mp4 /path/to/A.mp4

local msg = require "mp.msg"

local state = {
    a = nil,
    b = nil,
    mode = "a",
    single = "a",
    offset = 0.0,
    playlist_b = nil,
    keybar_visible = true,
}

local OSD_SHORT = 1.2
local OSD_LONG = 5.0
local status_overlay = mp.create_osd_overlay("ass-events")
local keybar_overlay = mp.create_osd_overlay("ass-events")

status_overlay.z = 20
keybar_overlay.z = 21

local function basename(path)
    if not path or path == "" then
        return nil
    end

    return path:match("([^/\\]+)$") or path
end

local function track_label(track, fallback)
    if not track then
        return fallback
    end

    return track.title
        or basename(track["external-filename"])
        or (fallback == "A" and basename(mp.get_property("path")))
        or string.format("%s (vid %s)", fallback, track.id or "?")
end

local function truncate(text, max_length)
    text = tostring(text or "")

    if #text <= max_length then
        return text
    end

    return text:sub(1, math.max(1, max_length - 3)) .. "..."
end

local function clean_number(value, decimals)
    local number = tonumber(value)

    if not number or number <= 0 then
        return nil
    end

    local formatted = string.format("%." .. tostring(decimals or 2) .. "f", number)
    formatted = formatted:gsub("0+$", ""):gsub("%.$", "")
    return formatted
end

local function track_spec(track)
    if not track then
        return "?"
    end

    local parts = {}
    local width = tonumber(track["demux-w"] or track["demux-crop-w"])
    local height = tonumber(track["demux-h"] or track["demux-crop-h"])

    if width and height and width > 0 and height > 0 then
        table.insert(parts, string.format("%dx%d", width, height))
    end

    local track_fps = clean_number(track["demux-fps"], 2)
    if track_fps then
        table.insert(parts, track_fps .. " fps")
    end

    local codec = track.codec or track["codec-desc"]
    if codec then
        local codec_text = codec
        if track["codec-profile"] then
            codec_text = codec_text .. " " .. track["codec-profile"]
        end
        table.insert(parts, codec_text)
    end

    if track["format-name"] then
        table.insert(parts, track["format-name"])
    end

    local bitrate = tonumber(track["demux-bitrate"])
    if bitrate and bitrate > 0 then
        table.insert(parts, clean_number(bitrate / 1000000, 1) .. " Mbps")
    end

    if #parts == 0 then
        return "?"
    end

    return table.concat(parts, " | ")
end

local function track_summary(track, fallback)
    return string.format("%s: %s  %s", fallback, truncate(track_label(track, fallback), 52), track_spec(track))
end

local function video_tracks()
    local list = mp.get_property_native("track-list") or {}
    local videos = {}

    for _, track in ipairs(list) do
        if track.type == "video" and not track.image and not track.albumart then
            table.insert(videos, track)
        end
    end

    table.sort(videos, function(left, right)
        return (left.id or 0) < (right.id or 0)
    end)

    return videos
end

local function ensure_tracks(show_error)
    local videos = video_tracks()

    if #videos < 2 then
        if show_error then
            mp.osd_message("A/B needs two videos. Open A B, or use --external-file=/path/to/B.mp4", OSD_LONG)
        end
        return false
    end

    state.a = videos[1]
    state.b = videos[2]
    return true
end

local function load_second_playlist_item()
    local playlist = mp.get_property_native("playlist") or {}
    local pos = mp.get_property_number("playlist-pos", 0)

    if #playlist ~= 2 or pos ~= 0 then
        return false
    end

    local second = playlist[2]
    local filename = second and second.filename

    if not filename or filename == "" or state.playlist_b == filename then
        return false
    end

    state.playlist_b = filename
    mp.commandv("video-add", filename, "auto", basename(filename) or "video")
    mp.commandv("playlist-remove", "1")

    return true
end

local function fps()
    local value = mp.get_property_number("container-fps")

    if (not value or value <= 0) and state.a then
        value = tonumber(state.a["demux-fps"])
    end

    if (not value or value <= 0) and state.b then
        value = tonumber(state.b["demux-fps"])
    end

    if not value or value <= 0 then
        value = 60
    end

    return value
end

local function clamp_time(value)
    local duration = mp.get_property_number("duration")

    if not value then
        return nil
    end

    if value < 0 then
        return 0
    end

    if duration and duration > 0 and value > duration then
        return duration
    end

    return value
end

local function logical_a_time()
    local now = mp.get_property_number("time-pos")

    if not now then
        return nil
    end

    if state.mode == "b" then
        return now - state.offset
    end

    return now
end

local function seek_exact(value)
    value = clamp_time(value)

    if value then
        mp.set_property_number("time-pos", value)
    end
end

local function clear_lavfi()
    if mp.get_property("lavfi-complex", "") ~= "" then
        mp.set_property("lavfi-complex", "")
    end
end

local function offset_text()
    if math.abs(state.offset) < 0.0005 then
        return "0f"
    end

    return string.format("%+.0ff (%+.3fs)", state.offset * fps(), state.offset)
end

local function ass_escape(text)
    text = tostring(text or "")
    text = text:gsub("\\", "\\\\")
    text = text:gsub("{", "\\{")
    text = text:gsub("}", "\\}")
    return text
end

local function mode_label(mode)
    local labels = {
        a = "A",
        b = "B",
        diff = "DIFF",
        stack = "SIDE",
        vstack = "TOP/BOTTOM",
        split = "SPLIT",
    }

    return labels[mode] or tostring(mode or "?"):upper()
end

local function status_text(prefix)
    return string.format(
        "%s%s | B offset: %s\n%s\n%s",
        prefix and (prefix .. "\n") or "",
        mode_label(state.mode),
        offset_text(),
        track_summary(state.a, "A"),
        track_summary(state.b, "B")
    )
end

local function title_summary(track, fallback, max_title)
    return string.format(
        "%s: %s  %s",
        fallback,
        truncate(track_label(track, fallback), max_title or 72),
        track_spec(track)
    )
end

local function media_title()
    if not state.a or not state.b then
        return nil
    end

    if state.mode == "a" then
        return title_summary(state.a, "A", 96)
    elseif state.mode == "b" then
        return title_summary(state.b, "B", 96)
    end

    return string.format(
        "%s  %s  |  %s",
        mode_label(state.mode),
        title_summary(state.a, "A", 42),
        title_summary(state.b, "B", 42)
    )
end

local function update_media_title()
    local title = media_title()

    if title then
        mp.set_property("force-media-title", title)
    end
end

local function update_overlay()
    status_overlay:remove()
    update_media_title()
end

local function key_item(key, label)
    local key_color = "{\\1c&H9CFF00&}"
    local text_color = "{\\1c&HFFFFFF&}"

    return key_color .. ass_escape(key) .. text_color .. " " .. ass_escape(label)
end

local function osd_visible()
    local level = mp.get_property_number("osd-level", 0)
    return level and level > 0
end

local function render_keybar()
    if not state.keybar_visible or not osd_visible() then
        keybar_overlay:remove()
        return
    end

    local sep = "{\\1c&H777777&} | {\\1c&HFFFFFF&}"
    local rows = {
        table.concat({
            key_item("ENTER/_/a", "flip"),
            key_item("TAB", "OSD"),
            key_item("1", "A"),
            key_item("2", "B"),
            key_item("[ ]", "nudge 1f"),
            key_item("{ }", "nudge 5f"),
            key_item("0", "reset"),
        }, sep),
        table.concat({
            key_item("l", "cycle layout"),
            key_item("v", "side"),
            key_item("t", "top/bottom"),
            key_item("s", "split"),
            key_item("d", "diff"),
            key_item("k", "keybar"),
            key_item("?", "help"),
        }, sep),
    }

    keybar_overlay.res_x = 1280
    keybar_overlay.res_y = 720
    keybar_overlay.data = table.concat({
        "{\\an7\\pos(12,570)\\bord0\\shad0\\1c&H000000&\\alpha&H80&\\p1}m 0 0 l 1256 0 l 1256 84 l 0 84{\\p0}",
        string.format(
            "{\\an7\\pos(24,586)\\bord2\\shad0\\fs22\\b1\\1c&HFFFFFF&\\3c&H000000&}%s",
            table.concat(rows, "\\N")
        ),
    }, "\n")
    keybar_overlay:update()
end

local function update_overlays()
    update_overlay()
    render_keybar()
end

local function show_status(prefix, duration)
    mp.osd_message(status_text(prefix), duration or OSD_SHORT)
    update_overlays()
    mp.commandv("show-progress")
end

local function select_side(side, quiet)
    if not ensure_tracks(true) then
        return
    end

    local base_time = logical_a_time()
    local target_time = base_time
    local target = state.a

    if side == "b" then
        target = state.b
        if base_time then
            target_time = base_time + state.offset
        end
    end

    clear_lavfi()
    mp.set_property_number("vid", target.id)

    if target_time and (side == "b" or state.mode == "b") and math.abs(state.offset) >= 0.0005 then
        seek_exact(target_time)
    end

    state.mode = side
    state.single = side
    update_overlays()

    if not quiet then
        show_status(side == "a" and "showing A" or "showing B")
    end
end

local function toggle()
    if state.single == "a" then
        select_side("b")
    else
        select_side("a")
    end
end

local function b_setpts()
    if math.abs(state.offset) < 0.0005 then
        return "setpts=PTS-STARTPTS"
    end

    if state.offset > 0 then
        return string.format("setpts=PTS-STARTPTS-%.6f/TB", state.offset)
    end

    return string.format("setpts=PTS-STARTPTS+%.6f/TB", -state.offset)
end

local function video_size(track)
    local width = tonumber(track and (track["demux-w"] or track["demux-crop-w"]))
    local height = tonumber(track and (track["demux-h"] or track["demux-crop-h"]))

    if width and height and width > 0 and height > 0 then
        return width, height
    end

    return nil, nil
end

local function comparison_graph(mode)
    local a_width, a_height = video_size(state.a)
    local a_chain = "setpts=PTS-STARTPTS,setsar=1"
    local b_chain = b_setpts()

    if mode == "diff" then
        if a_width and a_height then
            b_chain = string.format("%s,scale=%d:%d", b_chain, a_width, a_height)
        end
        b_chain = b_chain .. ",setsar=1"

        return string.format(
            "[vid%d]%s[a];[vid%d]%s[b];[a][b]blend=all_mode=difference[vo]",
            state.a.id,
            a_chain,
            state.b.id,
            b_chain
        )
    elseif mode == "stack" then
        if a_height then
            b_chain = string.format("%s,scale=-2:%d", b_chain, a_height)
        end
        b_chain = b_chain .. ",setsar=1"

        return string.format(
            "[vid%d]%s[a];[vid%d]%s[b];[a][b]hstack[vo]",
            state.a.id,
            a_chain,
            state.b.id,
            b_chain
        )
    elseif mode == "vstack" then
        if a_width then
            b_chain = string.format("%s,scale=%d:-2", b_chain, a_width)
        end
        b_chain = b_chain .. ",setsar=1"

        return string.format(
            "[vid%d]%s[a];[vid%d]%s[b];[a][b]vstack[vo]",
            state.a.id,
            a_chain,
            state.b.id,
            b_chain
        )
    elseif mode == "split" then
        if not (a_width and a_height) then
            return string.format(
                "[vid%d]%s,crop=iw/2:ih:0:0[left];[vid%d]%s,setsar=1,crop=iw/2:ih:iw/2:0[right];[left][right]hstack[vo]",
                state.a.id,
                a_chain,
                state.b.id,
                b_chain
            )
        end

        local half_width = math.floor(a_width / 2)
        local right_x = a_width - half_width

        return string.format(
            "[vid%d]setpts=PTS-STARTPTS,scale=%d:%d,setsar=1,crop=%d:%d:0:0[left];" ..
            "[vid%d]%s,scale=%d:%d,setsar=1,crop=%d:%d:%d:0[right];" ..
            "[left][right]hstack[vo]",
            state.a.id,
            a_width,
            a_height,
            half_width,
            a_height,
            state.b.id,
            b_chain,
            a_width,
            a_height,
            half_width,
            a_height,
            right_x
        )
    else
        return nil
    end
end

local function set_comparison_mode(mode)
    if not ensure_tracks(true) then
        return
    end

    if state.mode == mode then
        select_side(state.single)
        return
    end

    local graph = comparison_graph(mode)
    if not graph then
        return
    end

    local base_time = logical_a_time()
    mp.set_property("lavfi-complex", graph)
    if base_time then
        seek_exact(base_time)
    end

    state.mode = mode
    update_overlays()
    show_status(mode_label(mode) .. " view", OSD_LONG)
end

local layouts = { "stack", "vstack", "split", "diff" }

local function cycle_layout(direction)
    direction = direction or 1

    if not ensure_tracks(true) then
        return
    end

    local current = 0

    for index, layout in ipairs(layouts) do
        if state.mode == layout then
            current = index
            break
        end
    end

    local next_index = ((current + direction - 1) % #layouts) + 1
    set_comparison_mode(layouts[next_index])
end

local function nudge(frames)
    if not ensure_tracks(true) then
        return
    end

    local base_time = logical_a_time()
    state.offset = state.offset + (frames / fps())

    if state.mode == "b" and base_time then
        seek_exact(base_time + state.offset)
    elseif comparison_graph(state.mode) then
        mp.set_property("lavfi-complex", comparison_graph(state.mode))
        if base_time then
            seek_exact(base_time)
        end
    end

    show_status("nudged B")
end

local function reset_offset()
    local base_time = logical_a_time()
    state.offset = 0.0

    if state.mode == "b" and base_time then
        seek_exact(base_time)
    elseif comparison_graph(state.mode) then
        mp.set_property("lavfi-complex", comparison_graph(state.mode))
        if base_time then
            seek_exact(base_time)
        end
    end

    show_status("offset reset")
end

local function help()
    mp.osd_message(table.concat({
        "A/B compare",
        "ENTER, _, or a: flip A/B",
        "TAB: toggle OSD helpers",
        "1 / 2: show A / B",
        "l / L: cycle layouts",
        "v: side-by-side",
        "t: top/bottom",
        "s: split-screen",
        "d: difference",
        "[ / ]: nudge B by 1 frame",
        "{ / }: nudge B by 5 frames",
        "0: reset B offset",
        "k: toggle keybar",
    }, "\n"), OSD_LONG)
end

local function toggle_keybar()
    state.keybar_visible = not state.keybar_visible
    render_keybar()
end

local function set_osd_visible(visible)
    if visible then
        mp.set_property_number("osd-level", 1)
        mp.commandv("script-message-to", "osc", "osc-visibility", "always")
    else
        mp.set_property_number("osd-level", 0)
        mp.commandv("script-message-to", "osc", "osc-visibility", "never")
        status_overlay:remove()
        keybar_overlay:remove()
    end

    update_overlays()
end

local function toggle_osd()
    set_osd_visible(not osd_visible())
end

mp.register_event("file-loaded", function()
    if ensure_tracks(false) then
        select_side("a", true)
        msg.info("A/B compare ready")
        show_status("A/B compare ready", OSD_LONG)
    elseif load_second_playlist_item() then
        mp.add_timeout(0.1, function()
            if ensure_tracks(true) then
                select_side("a", true)
                msg.info("A/B compare ready")
                show_status("A/B compare ready", OSD_LONG)
            end
        end)
    else
        mp.osd_message("A/B compare: open A and B, or use --external-file=/path/to/B.mp4", OSD_LONG)
    end
end)

mp.observe_property("osd-dimensions", "native", update_overlays)
mp.observe_property("vid", "native", update_overlays)

mp.register_script_message("toggle", toggle)
mp.register_script_message("a", function() select_side("a") end)
mp.register_script_message("b", function() select_side("b") end)
mp.register_script_message("diff", function() set_comparison_mode("diff") end)
mp.register_script_message("stack", function() set_comparison_mode("stack") end)
mp.register_script_message("vstack", function() set_comparison_mode("vstack") end)
mp.register_script_message("split", function() set_comparison_mode("split") end)
mp.register_script_message("cycle-layout", function() cycle_layout(1) end)
mp.register_script_message("nudge", function(value) nudge(tonumber(value) or 1) end)
mp.register_script_message("reset-offset", reset_offset)
mp.register_script_message("help", help)

mp.add_key_binding(nil, "toggle", toggle)
mp.add_key_binding(nil, "show-a", function() select_side("a") end)
mp.add_key_binding(nil, "show-b", function() select_side("b") end)
mp.add_key_binding(nil, "diff", function() set_comparison_mode("diff") end)
mp.add_key_binding(nil, "stack", function() set_comparison_mode("stack") end)
mp.add_key_binding(nil, "vstack", function() set_comparison_mode("vstack") end)
mp.add_key_binding(nil, "split", function() set_comparison_mode("split") end)
mp.add_key_binding(nil, "cycle-layout", function() cycle_layout(1) end)
mp.add_key_binding(nil, "nudge-back", function() nudge(-1) end)
mp.add_key_binding(nil, "nudge-forward", function() nudge(1) end)
mp.add_key_binding(nil, "reset-offset", reset_offset)
mp.add_key_binding(nil, "help", help)
mp.add_key_binding(nil, "toggle-keybar", toggle_keybar)
mp.add_key_binding(nil, "toggle-osd", toggle_osd)

mp.add_forced_key_binding("TAB", "ab-toggle-osd", toggle_osd)
mp.add_forced_key_binding("ENTER", "ab-toggle-enter", toggle)
mp.add_forced_key_binding("KP_ENTER", "ab-toggle-kp-enter", toggle)
mp.add_forced_key_binding("_", "ab-toggle-underscore", toggle)
mp.add_forced_key_binding("a", "ab-toggle", toggle)
mp.add_forced_key_binding("1", "ab-show-a", function() select_side("a") end)
mp.add_forced_key_binding("2", "ab-show-b", function() select_side("b") end)
mp.add_forced_key_binding("d", "ab-diff", function() set_comparison_mode("diff") end)
mp.add_forced_key_binding("v", "ab-stack", function() set_comparison_mode("stack") end)
mp.add_forced_key_binding("t", "ab-vstack", function() set_comparison_mode("vstack") end)
mp.add_forced_key_binding("s", "ab-split", function() set_comparison_mode("split") end)
mp.add_forced_key_binding("l", "ab-cycle-layout", function() cycle_layout(1) end)
mp.add_forced_key_binding("L", "ab-cycle-layout-back", function() cycle_layout(-1) end)
mp.add_forced_key_binding("[", "ab-nudge-back", function() nudge(-1) end)
mp.add_forced_key_binding("]", "ab-nudge-forward", function() nudge(1) end)
mp.add_forced_key_binding("{", "ab-nudge-back-5", function() nudge(-5) end)
mp.add_forced_key_binding("}", "ab-nudge-forward-5", function() nudge(5) end)
mp.add_forced_key_binding("0", "ab-reset-offset", reset_offset)
mp.add_forced_key_binding("k", "ab-toggle-keybar", toggle_keybar)
mp.add_forced_key_binding("?", "ab-help-question", help)
mp.add_forced_key_binding("SHIFT+?", "ab-help", help)
