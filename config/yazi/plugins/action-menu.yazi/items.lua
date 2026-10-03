-- Entries for action-menu. Each group is a list of entries plus a `group`
-- label (shown in the left column, and searchable) and an optional `when`.
--
-- `when` applies to every target (the selection, or the hovered file):
--   kind   = "video" | "audio" | "image" | "dir" | "file", or a list of them
--   ext    = { "mp4", "m4v" }   -- files only, matched case-insensitively
--   single = true               -- exactly one target
--   or a function(targets) -> bool, where targets = { { name, ext, is_dir }, ... }
-- A group's `when` and an entry's `when` must both pass.
--
-- `run` is a Yazi shell command: %s = selected (or hovered), %s1 = first,
-- %d1 = its directory. Set `orphan = true` for anything that opens its own
-- window, `block = true` for anything that needs this terminal.

local BIN = "$HOME/.local/bin/"

-- kitty-launch wrapper used by most media tools: new kitty window/tab in %d1.
local function kitty(opts, title, cmd)
	return string.format('%skitty-launch %s --cwd %%d1 --title " %s " -- %s', BIN, opts, title, cmd)
end

-- Run a one-file-at-a-time tool over every selected file.
local function each(cmd) return string.format([[zsh -lc 'for f in "$@"; do %s "$f"; done' zsh %%s]], cmd) end

-- Run a one-file-at-a-time tool that takes explicit in/out paths, writing
-- "<name>_<suffix>.<ext>" next to each input.
local function eachOut(cmd, suffix, ext, extra)
	return string.format(
		[[zsh -lc 'for f in "$@"; do %s "$f" "${f:r}_%s.%s" %s; done' zsh %%s]],
		cmd, suffix, ext, extra or ""
	)
end

return { groups = {
	{
		group = "Topaz Video",
		when = { kind = "video" },
		{ desc = "􀰙  Enhance with FFMpeg (older models)", run = kitty("--tab --hold", "topaz", BIN .. "topaz-select-preset %s"), orphan = true },
		{ desc = "􀰙  Enhance with NeuroServer (newer models)", run = kitty("--tab --hold", "starlight", BIN .. "neuroserver-select-preset %s"), orphan = true },
	},
	{
		group = "Davinci Resolve",
		when = { kind = "video" },
		{ desc = "Interpolate: 60 fps Optical Flow, Speed Warp Metal, Medium", run = kitty("--window --hold", "resolve 60fps speed warp metal", BIN .. "interpolate-resolve --quality speed-warp-metal --range medium %s"), orphan = true },
		{ desc = "Interpolate: 60 fps Optical Flow, Speed Warp Metal, Medium (MKV)", run = kitty("--window --hold", "resolve 60fps speed warp metal mkv", BIN .. "interpolate-resolve --mkv --quality speed-warp-metal --range medium %s"), orphan = true },
		{ desc = "Interpolate: 60 fps Optical Flow, Enhanced Faster, Medium", run = kitty("--window --hold", "resolve 60fps enhanced faster", BIN .. "interpolate-resolve --quality enhanced-faster --range medium %s"), orphan = true },
		{ desc = "Interpolate: 60 fps Optical Flow, Speed Warp Metal, Wide", run = kitty("--window --hold", "resolve 60fps speed warp metal wide", BIN .. "interpolate-resolve --quality speed-warp-metal --range wide %s"), orphan = true },
		{ desc = "Interpolate: 60 fps Optical Flow, Speed Warp Metal, Wide (MKV)", run = kitty("--window --hold", "resolve 60fps speed warp metal wide mkv", BIN .. "interpolate-resolve --mkv --quality speed-warp-metal --range wide %s"), orphan = true },
		{ desc = "Interpolate: 60 fps Optical Flow, Enhanced Better, Wide", run = kitty("--window --hold", "resolve 60fps enhanced better wide", BIN .. "interpolate-resolve --quality enhanced-better --range wide %s"), orphan = true },
		{ desc = "Interpolate: 60 fps Optical Flow, Speed Warp Better, Medium", run = kitty("--window --hold", "resolve 60fps speed warp better", BIN .. "interpolate-resolve --quality speed-warp-better --range medium %s"), orphan = true },
	},
	{
		group = "Interpolation",
		when = { kind = "video" },
		{ desc = "60fps minterpolate", run = kitty("--window --hold", "smooth 60fps", each(BIN .. "smooth-fps")), orphan = true },
		{ desc = "120fps minterpolate", run = kitty("--window --hold", "smooth 120fps", each(BIN .. "smooth-fps --fps 120")), orphan = true },
		{ desc = "60fps minterpolate (dedup)", run = kitty("--window --hold", "smooth 60fps dedup", each(BIN .. "smooth-fps --dedup")), orphan = true },
		{ desc = "60fps rife", run = kitty("--window --hold", "rife 60fps", each(BIN .. "rife-vapoursynth")), orphan = true },
		{ desc = "60fps rife (dedup)", run = kitty("--window --hold", "rife 60fps dedup", each(BIN .. "rife-vapoursynth --dedup")), orphan = true },
	},
	{
		group = "Repair",
		when = { kind = "video" },
		{ desc = "Check For Duplicate Frames", run = kitty("--window --hold", "check duplicate frames", BIN .. "ffmpeg-check-fps %s"), orphan = true },
		{ desc = "Remove Duplicate Frames (Output: ProRes)", run = kitty("--window --hold", "discard duplicate frames", BIN .. "ffmpeg-discard-duplicate-frames %s"), orphan = true },
		{ desc = "Remove Duplicate Frames (Output: HEVC)", run = kitty("--window --hold", "discard duplicate frames", BIN .. "ffmpeg-discard-duplicate-frames --hevc %s"), orphan = true },
		{ desc = "Trim Intro", run = kitty("--window", "trim intro", BIN .. "ffmpeg-lossless-cut %s"), orphan = true },
		{ desc = "Trim Outro", run = kitty("--window", "trim outro", BIN .. "ffmpeg-lossless-cut --reverse %s"), orphan = true },
	},
	{
		group = "Apple ML",
		when = { kind = "video" },
		{ desc = "AVInterp: interpolate 2x fps (ProRes 422)", run = kitty("--window --hold", "avinterp 2x", eachOut(BIN .. "avinterp", "2x", "mov", "-c prores422")), orphan = true },
		{ desc = "AVInterp: 4x slow-mo (30fps)", run = kitty("--window --hold", "avinterp slowmo", eachOut(BIN .. "avinterp", "slowmo", "mov", "-f 4 --fps 30")), orphan = true },
		{ desc = "AVUpscale: super-res 4x (ProRes 422)", run = kitty("--window --hold", "avupscale 4x", eachOut(BIN .. "avupscale", "4x", "mov", "-s 4")), orphan = true },
		{ desc = "AVUpscale: super-res 4x (HEVC, 40mbps)", run = kitty("--window --hold", "avupscale 4x hevc", eachOut(BIN .. "avupscale", "4x", "mp4", "-s 4 -c hevc --mbps 40")), orphan = true },
		{ desc = "AVUpscale: list supported scales", run = kitty("--tab --hold", "avupscale --list", BIN .. "avupscale --list %s1 /dev/null"), orphan = true, when = { single = true } },
		{ desc = "AVRemove: check mask (copies template command)", run = kitty("--window --hold", "avremove mask-check", [[zsh -lc 'cmd="avremove $1 mask-check.mov --point X,Y --debug-mask"; print -r -- "$cmd" | pbcopy; echo "$cmd"; echo "(copied to clipboard - paste, fix --point, then run)"; exec zsh' zsh %s1]]), orphan = true, when = { single = true } },
		{ desc = "AVRemove: remove object (copies template command)", run = kitty("--window --hold", "avremove", [[zsh -lc 'cmd="avremove $1 out.mov --point X,Y --model LaMa512.mlpackage"; print -r -- "$cmd" | pbcopy; echo "$cmd"; echo "(copied to clipboard - paste, fix --point, then run)"; exec zsh' zsh %s1]]), orphan = true, when = { single = true } },
	},
	{
		group = "Video",
		when = { kind = "video" },
		{ desc = "Edit Tags", run = kitty("--tab", "tagform", BIN .. "tagform %s"), orphan = true },
		{ desc = "Rename Based On Metadata Tags", run = kitty("--window --hold", "rename video", BIN .. "rename-video --color --icons %s"), orphan = true },
		{ desc = "Media Audit", run = kitty("--tab --hold", "media-audit", BIN .. "media-audit-batch %s"), orphan = true },
		{ desc = "Autodetect Chapters", run = kitty("--window --hold", "chapters", BIN .. "ffmpeg-scenedetect-chapters %s"), orphan = true },
		{ desc = "Rotate Right (lossless)",run = BIN .. "video-rotate-lossless --right %s", block = true },
		{ desc = "Rotate Left (lossless)", run = BIN .. "video-rotate-lossless --left %s", block = true },
		{ desc = "Convert To ProRes", run = "convert-to-pro-res %s .", orphan = true },
		{ desc = "Convert To HEVC", run = "convert-to-hevc %s .", orphan = true },
		{ desc = "MediaInfo", run = "mediainfo %s1; echo 'Press enter to exit'; read _", block = true, when = { single = true } },
	},
	{
		group = "AV1",
		when = { kind = "video" },
		{ desc = "Encode AV1: Fast (bulk / preview)", run = kitty("--window --hold", "av1 fast", BIN .. "encode-av1 --fast %s"), orphan = true },
		{ desc = "Encode AV1: Balanced (default)", run = kitty("--window --hold", "av1 balanced", BIN .. "encode-av1 --balanced %s"), orphan = true },
		{ desc = "Encode AV1: Quality (fine detail, slow)", run = kitty("--window --hold", "av1 quality", BIN .. "encode-av1 --quality %s"), orphan = true },
		{ desc = "Encode AV1: Archive (near-transparent, very slow)", run = kitty("--window --hold", "av1 archive", BIN .. "encode-av1 --archive %s"), orphan = true },
	},
	{
		group = "MP4",
		when = { ext = { "mp4", "m4v", "mov" } },
		{ desc = "MP4Doctor", run = kitty("--tab --hold", "mp4doctor", BIN .. "mp4doctor %s"), orphan = true },
		{ desc = "MP4Doctor --fix", run = kitty("--tab --hold", "mp4doctor fix", BIN .. "mp4doctor --fix %s"), orphan = true },
		{ desc = "Enable FastStart", run = kitty("--window --hold", "faststart", BIN .. "fast-start enable %s"), orphan = true },
		{ desc = "Convert to MKV", run = kitty("--window", "to-mkv", BIN .. "to-mkv %s"), orphan = true },
	},
	{
		group = "MOV",
		when = { ext = { "mov" } },
		{ desc = "Convert to MP4", run = kitty("--window", "to-mp4", BIN .. "to-mp4 %s"), orphan = true, when = { ext = { "mov" } } },
	},
	{
		group = "Play",
		when = { kind = { "video", "dir" } },
		{ desc = "Open in MPV", run = "mpv --force-window --fullscreen --no-native-fs %s", orphan = true },
		{ desc = "Send to MPV (Socket)", run = "mpv-send play %s", orphan = true },
		{ desc = "Open in Switchblade", run = "open -a Switchblade %s", orphan = true },
		{ desc = "Open in Abner", run = "open -a Abner %s", orphan = true },
		{ desc = "Open in VLC", run = "vlc %s .", orphan = true },
	},
	{
		group = "Folder",
		when = { kind = "dir" },
		{ desc = "Media Audit", run = kitty("--tab --hold", "check media", BIN .. "media-audit-batch %s"), orphan = true },
		{ desc = "Open in MP4Doctor", run = kitty("--tab --hold", "mp4doctor", BIN .. "mp4doctor %s"), orphan = true },
		{ desc = "Reveal in Finder", run = "open %s" },
		{ desc = "Serve at http://localhost:8000", run = "python3 -m http.server 8000 && open http://localhost:8000", orphan = true },
	},
	{
		group = "Subtitles",
		when = { ext = { "vtt" } },
		-- Video is "<name>.<ext>" next to the .vtt ("<name>.en.vtt" also matches). MP4/MOV
		-- can't carry WebVTT, so those get mov_text. On success the original video and the .vtt go to the Trash and the new file takes the video's name.
		{ desc = "Embed .vtt subtitles with ffmpeg", run = kitty("--hold-on-fail", "embed subtitles", [==[zsh -lc 'for s in "$@"; do v=; for b in "${s:r}" "${s:r:r}"; do for e in mkv mp4 m4v mov webm; do [[ -f $b.$e ]] && v=$b.$e && break 2; done; done; [[ -n $v ]] || { echo "no video for: $s"; continue }; c=webvtt; [[ ${v:e:l} == (mp4|m4v|mov) ]] && c=mov_text; ffmpeg -i "$v" -i "$s" -c copy -c:s $c "${v:r}_subs.${v:e}" && trash "$v" "$s" && mv "${v:r}_subs.${v:e}" "$v"; done' zsh %s]==]), orphan = true },
	},
	{
		group = "Audio",
		when = { kind = "audio" },
		{ desc = "Neural Mix: track report (temporary)", run = kitty("--tab --hold", "Neural Mix report", BIN .. "yazi-neural-mix-report %s"), orphan = true },
		{ desc = "FLAC Stems Sidecar", run = kitty("--window --hold", "lossless flac sidecar", each(BIN .. "vdjstems make --roformer --flac")), orphan = true, when = { ext = { "flac" } } },
	},
	{
		group = "File",
		{ desc = "Open (Default)", run = "open %s" },
		{ desc = "Reveal in Finder", run = "open -R %s1", when = { single = true } },
		{ desc = "Drag Out", run = "kitten dnd %s" },
		{ desc = "File", run = "clear; file %s1; echo 'Press enter to exit'; read _", block = true, when = { single = true } },
		{ desc = "MediaInfo", run = "clear; mediainfo %s1; echo 'Press enter to exit'; read _", block = true, when = { single = true } },
		{ desc = "EXIFTool", run = "clear; exiftool %s1; echo 'Press enter to exit'; read _", block = true, when = { single = true } },
		{ desc = "safesync lookup", run = "clear; safesync lookup --file %s1; echo 'Press enter to exit'; read _", block = true, when = { single = true } },
	},
} }
