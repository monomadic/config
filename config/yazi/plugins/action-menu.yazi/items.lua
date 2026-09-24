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
		{ desc = "Interpolate: 60 fps Optical Flow, Enhanced Faster, Medium", run = kitty("--window --hold", "resolve 60fps enhanced faster", BIN .. "interpolate-resolve --quality enhanced-faster --range medium %s"), orphan = true },
		{ desc = "Interpolate: 60 fps Optical Flow, Speed Warp Metal, Wide", run = kitty("--window --hold", "resolve 60fps speed warp metal wide", BIN .. "interpolate-resolve --quality speed-warp-metal --range wide %s"), orphan = true },
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
		{ desc = "Check Duplicate Frames", run = kitty("--window --hold", "check duplicate frames", BIN .. "ffmpeg-check-fps %s"), orphan = true },
		{ desc = "Remove Duplicate Frames (Output: ProRes)", run = kitty("--window --hold", "discard duplicate frames", BIN .. "ffmpeg-discard-duplicate-frames %s"), orphan = true },
		{ desc = "Remove Duplicate Frames (Output: HEVC)", run = kitty("--window --hold", "discard duplicate frames", BIN .. "ffmpeg-discard-duplicate-frames --hevc %s"), orphan = true },
		{ desc = "Trim Intro", run = kitty("--window", "trim intro", BIN .. "ffmpeg-lossless-cut %s"), orphan = true },
		{ desc = "Trim Outro", run = kitty("--window", "trim outro", BIN .. "ffmpeg-lossless-cut --reverse %s"), orphan = true },
	},
	{
		group = "Video",
		when = { kind = "video" },
		{ desc = "rename (tags)", run = kitty("--window --hold", "rename video", BIN .. "rename-video --color --icons %s"), orphan = true },
		{ desc = "rename footage", run = kitty("--window", "rename footage", BIN .. "rename-footage %s"), orphan = true },
		{ desc = "Edit Tags", run = kitty("--tab", "tagform", BIN .. "tagform %s"), orphan = true },
		{ desc = "tag from url", run = kitty("--window --hold", "tag from url", each(BIN .. "yt-dlp-refresh-metadata")), orphan = true },
		{ desc = "set star rating", run = kitty("--window", "set rating", BIN .. "media-set-rating %s"), orphan = true },
		{ desc = "media audit", run = kitty("--tab --hold", "media-audit", BIN .. "media-audit-batch %s"), orphan = true },
		{ desc = "autodetect chapters", run = kitty("--window --hold", "chapters", BIN .. "ffmpeg-scenedetect-chapters %s"), orphan = true },
		{ desc = "rotate right (lossless)", run = BIN .. "video-rotate-lossless --right %s", block = true },
		{ desc = "rotate left (lossless)", run = BIN .. "video-rotate-lossless --left %s", block = true },
		{ desc = "convert to prores", run = "convert-to-pro-res %s .", orphan = true },
		{ desc = "convert to hevc", run = "convert-to-hevc %s .", orphan = true },
		{ desc = "mediainfo", run = "mediainfo %s1; echo 'Press enter to exit'; read _", block = true, when = { single = true } },
	},
	{
		group = "MP4",
		when = { ext = { "mp4", "m4v", "mov" } },
		{ desc = "mp4doctor", run = kitty("--tab --hold", "mp4doctor", BIN .. "mp4doctor %s"), orphan = true },
		{ desc = "mp4doctor --fix", run = kitty("--tab --hold", "mp4doctor fix", BIN .. "mp4doctor --fix %s"), orphan = true },
		{ desc = "enable faststart", run = kitty("--window --hold", "faststart", BIN .. "fast-start enable %s"), orphan = true },
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
		{ desc = "Send to MPV (via Socket)", run = "mpv-send play %s", orphan = true, when = { kind = "video" } },
		{ desc = "Open in Switchblade", run = "open -a Switchblade %s", orphan = true },
		{ desc = "Open in a/bner", run = "open -a Abner %s", orphan = true, when = { kind = "video" } },
		{ desc = "Open in VLC", run = "vlc %s .", orphan = true },
	},
	{
		group = "Folder",
		when = { kind = "dir" },
		{ desc = "Media Audit", run = kitty("--tab --hold", "check media", BIN .. "media-audit-batch %s"), orphan = true },
		{ desc = "Open in MP4Doctor", run = kitty("--tab --hold", "mp4doctor", BIN .. "mp4doctor %s"), orphan = true },
		{ desc = "Reveal in Finder", run = "open %s" },
	},
	{
		group = "Subtitles",
		when = { ext = { "vtt" } },
		{ desc = "Embed into MP4", run = kitty("--window --hold", "embed subtitles", each(BIN .. "mp4-embed-vtt")), orphan = true },
	},
	{
		group = "Audio",
		when = { ext = { "flac" } },
		{ desc = "FLAC Stems Sidecar", run = kitty("--window --hold", "lossless flac sidecar", each(BIN .. "vdjstems make --roformer --flac")), orphan = true },
	},
	{
		group = "File",
		{ desc = "Open (Default)", run = "open %s" },
		{ desc = "Reveal in Finder", run = "open -R %s1", when = { single = true } },
		{ desc = "Drag Out", run = "kitten dnd %s" },
		{ desc = "File", run = "clear; file %s1; echo 'Press enter to exit'; read _", block = true, when = { single = true } },
		{ desc = "MediaInfo", run = "clear; mediainfo %s1; echo 'Press enter to exit'; read _", block = true, when = { single = true } },
		{ desc = "EXIFTool", run = "clear; exiftool %s1; echo 'Press enter to exit'; read _", block = true, when = { single = true } },
	},
} }
