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
		group = "Topaz",
		when = { kind = "video" },
		{ desc = "workflow", run = kitty("--window --hold", "topaz workflow", BIN .. "topaz-workflow %s"), orphan = true },
		{ desc = "simple presets", run = kitty("--window --hold", "topaz presets", BIN .. "topaz-simple-presets %s"), orphan = true },
		{ desc = "select preset: preview + encode (ffmpeg)", run = kitty("--tab --hold", "topaz", BIN .. "topaz-select-preset %s"), orphan = true },
		{ desc = "select preset: starlight (neuroserver)", run = kitty("--tab --hold", "starlight", BIN .. "neuroserver-select-preset %s"), orphan = true },
	},
	{
		group = "Interpolate",
		when = { kind = "video" },
		{ desc = "60fps resolve (hevc)", run = kitty("--window --hold", "resolve 60fps", BIN .. "interpolate-resolve %s"), orphan = true },
		{ desc = "60fps resolve (prores)", run = kitty("--window --hold", "resolve 60fps prores", BIN .. "interpolate-resolve --prores %s"), orphan = true },
		{ desc = "120fps resolve (hevc)", run = kitty("--window --hold", "resolve 120fps", BIN .. "interpolate-resolve 120 %s"), orphan = true },
		{ desc = "120fps resolve (prores)", run = kitty("--window --hold", "resolve 120fps prores", BIN .. "interpolate-resolve --prores 120 %s"), orphan = true },
		{ desc = "60fps resolve job (queue)", run = BIN .. "interpolate-resolve-job %s", block = true },
		{ desc = "120fps resolve job (queue)", run = BIN .. "interpolate-resolve-job 120 %s", block = true },
		{ desc = "60fps minterpolate", run = kitty("--window --hold", "smooth 60fps", each(BIN .. "smooth-fps")), orphan = true },
		{ desc = "120fps minterpolate", run = kitty("--window --hold", "smooth 120fps", each(BIN .. "smooth-fps --fps 120")), orphan = true },
		{ desc = "60fps minterpolate (dedup)", run = kitty("--window --hold", "smooth 60fps dedup", each(BIN .. "smooth-fps --dedup")), orphan = true },
		{ desc = "60fps rife", run = kitty("--window --hold", "rife 60fps", each(BIN .. "rife-vapoursynth")), orphan = true },
		{ desc = "60fps rife (dedup)", run = kitty("--window --hold", "rife 60fps dedup", each(BIN .. "rife-vapoursynth --dedup")), orphan = true },
	},
	{
		group = "Trim",
		when = { kind = "video" },
		{ desc = "intro (start)", run = kitty("--window", "trim intro", BIN .. "ffmpeg-lossless-cut %s"), orphan = true },
		{ desc = "outro (end)", run = kitty("--window", "trim outro", BIN .. "ffmpeg-lossless-cut --reverse %s"), orphan = true },
	},
	{
		group = "Video",
		when = { kind = "video" },
		{ desc = "rename (tags)", run = kitty("--window --hold", "rename video", BIN .. "rename-video --color --icons %s"), orphan = true },
		{ desc = "rename footage", run = kitty("--window", "rename footage", BIN .. "rename-footage %s"), orphan = true },
		{ desc = "edit tags", run = kitty("--tab", "tagform", BIN .. "tagform %s"), orphan = true },
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
		{ desc = "mov → mp4", run = kitty("--window", "to-mp4", BIN .. "to-mp4 %s"), orphan = true, when = { ext = { "mov" } } },
	},
	{
		group = "Play",
		when = { kind = { "video", "dir" } },
		{ desc = "mpv", run = "mpv --force-window --fullscreen --no-native-fs %s", orphan = true },
		{ desc = "mpv (send to running)", run = "mpv-send play %s", orphan = true, when = { kind = "video" } },
		{ desc = "switchblade", run = "open -a Switchblade %s", orphan = true },
		{ desc = "a/bner", run = "open -a Abner %s", orphan = true, when = { kind = "video" } },
		{ desc = "vlc", run = "vlc %s .", orphan = true },
	},
	{
		group = "Folder",
		when = { kind = "dir" },
		{ desc = "media audit", run = kitty("--tab --hold", "check media", BIN .. "media-audit-batch %s"), orphan = true },
		{ desc = "mp4doctor", run = kitty("--tab --hold", "mp4doctor", BIN .. "mp4doctor %s"), orphan = true },
		{ desc = "open in Finder", run = "open %s" },
	},
	{
		group = "Subtitles",
		when = { ext = { "vtt" } },
		{ desc = "embed into mp4", run = kitty("--window --hold", "embed subtitles", each(BIN .. "mp4-embed-vtt")), orphan = true },
	},
	{
		group = "Audio",
		when = { ext = { "flac" } },
		{ desc = "flac stems sidecar", run = kitty("--window --hold", "lossless flac sidecar", each(BIN .. "vdjstems make --roformer --flac")), orphan = true },
	},
	{
		group = "File",
		{ desc = "open (default app)", run = "open %s" },
		{ desc = "reveal in Finder", run = "open -R %s1", when = { single = true } },
		{ desc = "drag out", run = "kitten dnd %s" },
		{ desc = "exiftool", run = "clear; exiftool %s1; echo 'Press enter to exit'; read _", block = true, when = { single = true } },
	},
} }
