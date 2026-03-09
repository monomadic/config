local mp = require "mp"
local utils = require "mp.utils"

local RECURSIVE = true
local SKIP_DOTFILES = true
local OSD_SECS = 4

local VIDEO_EXT = {
  ["mp4"]=true, ["mkv"]=true, ["webm"]=true, ["mov"]=true, ["m4v"]=true,
  ["avi"]=true, ["mpg"]=true, ["mpeg"]=true, ["m2ts"]=true, ["mts"]=true,
  ["ts"]=true, ["vob"]=true, ["wmv"]=true, ["flv"]=true, ["f4v"]=true,
  ["3gp"]=true, ["3g2"]=true, ["ogv"]=true
}

local function osd(msg, secs)
  mp.osd_message(msg, secs or OSD_SECS)
  mp.msg.warn(msg)
end

local function lower(s)
  return (s and s:lower()) or s
end

local function is_nonfile_url(p)
  return type(p) == "string"
    and p:match("^[%a][%w+.-]*://")
    and not p:match("^file://")
end

local function uri_decode(s)
  return (s:gsub("%%(%x%x)", function(h)
    return string.char(tonumber(h, 16))
  end))
end

local function file_uri_to_path(u)
  if type(u) ~= "string" then return u end
  if not u:match("^file://") then return u end

  local p = u:gsub("^file://", "")
  p = p:gsub("^localhost", "")
  if p == "" then p = "/" end
  return uri_decode(p)
end

local function normalize_path(p)
  if not p then return nil end
  if is_nonfile_url(p) then return p end
  return file_uri_to_path(p)
end

local function ext_of(path)
  if not path then return nil end
  local p = path:match("([^/?#]+)") or path
  local e = p:match("%.([^.]+)$")
  return lower(e)
end

local function is_video_path(path)
  if is_nonfile_url(path) then return true end
  local e = ext_of(path)
  return e and VIDEO_EXT[e] or false
end

local function wd_join(p)
  if not p or is_nonfile_url(p) then return p end
  p = normalize_path(p)
  if p:sub(1,1) == "/" then return p end
  local wd = mp.get_property("working-directory") or "."
  return utils.join_path(wd, p)
end

local function get_info(p)
  if not p or is_nonfile_url(p) then return nil end
  return utils.file_info(p)
end

local function get_mtime(p)
  if is_nonfile_url(p) then return -1 end
  local abs = wd_join(p)
  local info = utils.file_info(abs)
  if not info or not info.mtime then
    mp.msg.warn(("mtime missing: %s"):format(tostring(abs)))
    return -1
  end
  return info.mtime
end

local function readdir_safe(dir, what)
  local t = utils.readdir(dir, what)
  if not t then
    mp.msg.warn(("readdir nil: dir=%s what=%s"):format(tostring(dir), tostring(what)))
    return {}
  end
  return t
end

local function should_skip_name(name)
  return SKIP_DOTFILES and name:sub(1,1) == "."
end

local function scan_dir(dir_abs, out, seen, depth)
  depth = depth or 0
  osd(("scan dir depth=%d:\n%s"):format(depth, dir_abs), 2)

  local files = readdir_safe(dir_abs, "files")
  local dirs  = readdir_safe(dir_abs, "dirs")

  mp.msg.warn(("scan_dir: %s | files=%d dirs=%d"):format(dir_abs, #files, #dirs))

  for _, name in ipairs(files) do
    local p = utils.join_path(dir_abs, name)
    local skip = should_skip_name(name)
    local video = is_video_path(p)
    local dup = seen[p] and true or false

    mp.msg.warn(("  file: %s | skip=%s video=%s dup=%s"):format(
      p, tostring(skip), tostring(video), tostring(dup)
    ))

    if not skip and video and not dup then
      seen[p] = true
      out[#out + 1] = p
      osd(("add file:\n%s"):format(name), 1.5)
    end
  end

  if RECURSIVE then
    for _, name in ipairs(dirs) do
      local p = utils.join_path(dir_abs, name)
      local skip = should_skip_name(name)
      mp.msg.warn(("  dir: %s | skip=%s"):format(p, tostring(skip)))
      if not skip then
        scan_dir(p, out, seen, depth + 1)
      end
    end
  end
end

local function collect_playlist_entries(expand_dirs)
  local pl = mp.get_property_native("playlist")
  if not pl or #pl == 0 then
    osd("playlist: empty", 4)
    return nil, false
  end

  local files = {}
  local seen = {}
  local expanded_any = false

  osd(("playlist entries: %d"):format(#pl), 2)

  for i, it in ipairs(pl) do
    local raw = it.filename
    local current = it.current and true or false
    local playing = it.playing and true or false

    mp.msg.warn(("playlist[%d] raw=%s current=%s playing=%s"):format(
      i, tostring(raw), tostring(current), tostring(playing)
    ))

    if not raw then
      mp.msg.warn(("playlist[%d] skipped: no filename"):format(i))
      goto continue
    end

    if is_nonfile_url(raw) then
      if not seen[raw] then
        seen[raw] = true
        files[#files + 1] = raw
        osd(("keep URL:\n%s"):format(raw), 1.25)
      end
      goto continue
    end

    local abs = wd_join(raw)
    local info = get_info(abs)

    mp.msg.warn(("playlist[%d] abs=%s"):format(i, tostring(abs)))
    mp.msg.warn(("playlist[%d] info=%s"):format(i, info and "yes" or "no"))

    if not info then
      mp.msg.warn(("playlist[%d] rejected: file_info nil for %s"):format(i, tostring(abs)))
      goto continue
    end

    mp.msg.warn(("playlist[%d] info.is_dir=%s size=%s mtime=%s"):format(
      i, tostring(info.is_dir), tostring(info.size), tostring(info.mtime)
    ))

    if info.is_dir then
      if expand_dirs then
        local before = #files
        osd(("expand dir:\n%s"):format(abs), 2)
        scan_dir(abs, files, seen, 0)
        if #files > before then
          expanded_any = true
        end
      else
        mp.msg.warn(("playlist[%d] skipping dir (expand disabled): %s"):format(i, abs))
      end
    else
      -- IMPORTANT: if mpv already has it in the playlist and it is a real file,
      -- keep it even if extension detection fails.
      if not seen[abs] then
        seen[abs] = true
        files[#files + 1] = abs
        osd(("keep file:\n%s"):format(abs), 1.25)
      else
        mp.msg.warn(("playlist[%d] duplicate file skipped: %s"):format(i, abs))
      end
    end

    ::continue::
  end

  mp.msg.warn(("collect_playlist_entries: total=%d expanded_any=%s"):format(
    #files, tostring(expanded_any)
  ))
  for i, p in ipairs(files) do
    mp.msg.warn(("collected[%d]=%s"):format(i, tostring(p)))
  end

  return files, expanded_any
end

local function sort_paths_by_mtime(paths)
  local items = {}
  for i, p in ipairs(paths) do
    local mt = get_mtime(p)
    items[i] = { path = p, mtime = mt, idx = i }
    mp.msg.warn(("pre-sort[%d] mtime=%s path=%s"):format(i, tostring(mt), tostring(p)))
  end

  table.sort(items, function(a, b)
    if a.mtime == b.mtime then
      return a.idx < b.idx
    end
    return a.mtime > b.mtime
  end)

  local sorted = {}
  for i, it in ipairs(items) do
    sorted[i] = it.path
    mp.msg.warn(("post-sort[%d] mtime=%s path=%s"):format(i, tostring(it.mtime), tostring(it.path)))
  end

  return sorted
end

local function reload_playlist(paths, play_first)
  if not paths or #paths == 0 then
    osd("playlist: nothing to reload", 4)
    return
  end

  if play_first then
    osd(("replace current with:\n%s"):format(paths[1]), 3)
    mp.commandv("loadfile", paths[1], "replace")
    for i = 2, #paths do
      mp.msg.warn(("append[%d]: %s"):format(i, tostring(paths[i])))
      mp.commandv("loadfile", paths[i], "append")
    end
    mp.commandv("playlist-play-index", "0")
    mp.set_property_bool("pause", false)
    osd(("reloaded %d items\nplaying:\n%s"):format(#paths, paths[1]), 4)
  else
    local was_paused = mp.get_property_bool("pause")
    mp.commandv("playlist-clear")
    for i, p in ipairs(paths) do
      mp.msg.warn(("append[%d]: %s"):format(i, tostring(p)))
      mp.commandv("loadfile", p, "append")
    end
    mp.set_property_bool("pause", was_paused)
    osd(("reloaded playlist: %d items"):format(#paths), 3)
  end
end

local function action_sort_playlist()
  local files = select(1, collect_playlist_entries(false))
  if not files or #files == 0 then
    osd("sort: collector returned 0 files", 4)
    return
  end

  local sorted = sort_paths_by_mtime(files)
  reload_playlist(sorted, false)
  osd(("sorted playlist by mtime: %d items"):format(#sorted), 4)
end

local function action_expand_playlist()
  local files, expanded_any = collect_playlist_entries(true)
  if not files or #files == 0 then
    osd("expand: collector returned 0 files", 4)
    return
  end

  reload_playlist(files, false)

  if expanded_any then
    osd(("expanded playlist: %d items"):format(#files), 4)
  else
    osd(("expand: no new files found, kept %d items"):format(#files), 4)
  end
end

local function action_expand_sort_play_newest()
  local files, expanded_any = collect_playlist_entries(true)
  if not files or #files == 0 then
    osd("expand+sort+play: collector returned 0 files", 4)
    return
  end

  local sorted = sort_paths_by_mtime(files)
  reload_playlist(sorted, true)

  if expanded_any then
    osd(("expanded + sorted + playing newest:\n%s"):format(sorted[1] or "?"), 5)
  else
    osd(("no expansion needed; sorted + playing newest:\n%s"):format(sorted[1] or "?"), 5)
  end
end

mp.add_key_binding("Ctrl+s", "sort_playlist_by_mtime", action_sort_playlist)
mp.add_key_binding("Ctrl+d", "expand_playlist_dirs", action_expand_playlist)
mp.add_key_binding("Ctrl+e", "expand_sort_play_newest", action_expand_sort_play_newest)
