local mp = require "mp"
local utils = require "mp.utils"

-- ===== config =====
local RECURSIVE = true
local SKIP_DOTFILES = true

-- whitelist (lowercase extensions, no dot)
local VIDEO_EXT = {
  ["mp4"]=true, ["mkv"]=true, ["webm"]=true, ["mov"]=true, ["m4v"]=true,
  ["avi"]=true, ["mpg"]=true, ["mpeg"]=true, ["m2ts"]=true, ["mts"]=true,
  ["ts"]=true, ["vob"]=true, ["wmv"]=true, ["flv"]=true, ["f4v"]=true,
  ["3gp"]=true, ["3g2"]=true, ["ogv"]=true
}
-- ================

local function is_url(p)
  return type(p) == "string" and p:match("^[%a][%w+.-]*://")
end

local function lower(s) return (s and s:lower()) or s end

local function ext_of(path)
  if not path then return nil end
  local p = path:match("([^/?#]+)") or path
  local e = p:match("%.([^.]+)$")
  return lower(e)
end

local function is_video_path(path)
  if is_url(path) then return true end -- keep URLs
  local e = ext_of(path)
  return e and VIDEO_EXT[e] or false
end

local function wd_join(p)
  if not p or is_url(p) then return p end
  if p:sub(1,1) == "/" then return p end
  local wd = mp.get_property("working-directory") or "."
  return utils.join_path(wd, p)
end

local function file_info_abs(p_abs)
  if not p_abs or is_url(p_abs) then return nil end
  return utils.file_info(p_abs)
end

local function is_dir_path(p_abs)
  local info = file_info_abs(p_abs)
  return info and info.is_dir
end

local function get_mtime(p)
  if is_url(p) then return -1 end
  local abs = wd_join(p)
  local info = utils.file_info(abs)
  if not info or not info.mtime then return -1 end
  return info.mtime
end

local function readdir_safe(dir, what)
  local t = utils.readdir(dir, what)
  return t or {}
end

local function should_skip_name(name)
  return SKIP_DOTFILES and name:sub(1,1) == "."
end

local function scan_dir(dir_abs, out, seen)
  for _, name in ipairs(readdir_safe(dir_abs, "files")) do
    if not should_skip_name(name) then
      local p = utils.join_path(dir_abs, name)
      if not seen[p] and is_video_path(p) then
        seen[p] = true
        out[#out + 1] = p
      end
    end
  end
  if RECURSIVE then
    for _, name in ipairs(readdir_safe(dir_abs, "dirs")) do
      if not should_skip_name(name) then
        scan_dir(utils.join_path(dir_abs, name), out, seen)
      end
    end
  end
end

local function sort_and_reload(paths)
  local items = {}
  for i, p in ipairs(paths) do
    items[i] = { path = p, mtime = get_mtime(p), idx = i }
  end
  table.sort(items, function(a, b)
    if a.mtime == b.mtime then return a.idx < b.idx end
    return a.mtime > b.mtime
  end)

  mp.commandv("playlist-clear")
  for _, it in ipairs(items) do
    mp.commandv("loadfile", it.path, "append-play")
  end

  mp.commandv("playlist-play-index", "0")
  mp.set_property_bool("pause", false)
  mp.osd_message(("sorted by mtime: %d items"):format(#items))
end

local function expand_dirs_sort_mtime_play_newest()
  local pl = mp.get_property_native("playlist")
  if not pl or #pl == 0 then
    mp.osd_message("playlist: empty")
    return
  end

  local files = {}
  local seen = {}

  -- First pass: expand dirs + include existing video items
  for _, it in ipairs(pl) do
    local fn = it.filename
    if not fn then goto continue end

    if is_url(fn) then
      -- keep URLs
      if not seen[fn] then
        seen[fn] = true
        files[#files + 1] = fn
      end
      goto continue
    end

    local abs = wd_join(fn)
    local info = utils.file_info(abs)

    if info and info.is_dir then
      scan_dir(abs, files, seen)
    else
      if is_video_path(abs) and not seen[abs] then
        seen[abs] = true
        files[#files + 1] = abs
      end
    end

    ::continue::
  end

  -- If expanding dirs produced *no* video files, fallback:
  -- sort whatever is already in playlist that isn't a directory (and keep URLs),
  -- so you still get a sorted playlist and playback.
  if #files == 0 then
    local fallback = {}
    local fseen = {}
    for _, it in ipairs(pl) do
      local fn = it.filename
      if not fn then goto cont2 end

      if is_url(fn) then
        if not fseen[fn] then
          fseen[fn] = true
          fallback[#fallback + 1] = fn
        end
        goto cont2
      end

      local abs = wd_join(fn)
      local info = utils.file_info(abs)
      if info and info.is_dir then
        goto cont2
      end

      if not fseen[abs] then
        fseen[abs] = true
        fallback[#fallback + 1] = abs
      end

      ::cont2::
    end

    if #fallback == 0 then
      mp.osd_message("playlist: nothing sortable/playable")
      return
    end

    sort_and_reload(fallback)
    return
  end

  sort_and_reload(files)
end

mp.add_key_binding("CTRL+e", "expand_dirs_sort_mtime_play_newest", expand_dirs_sort_mtime_play_newest)
