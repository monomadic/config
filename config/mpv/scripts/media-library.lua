-- media-library.lua
-- CMD+A: replace playlist with output of ls-media

mp.add_key_binding(nil, "load-all-media", function()
    local handle = io.popen("/Users/nom/.zsh/bin/ls-media")
    if not handle then
        mp.osd_message("load-all-media: failed to run ls-media", 3)
        return
    end

    local files = {}
    for line in handle:lines() do
        local f = line:match("^(.-)%s*$")  -- trim trailing whitespace/CR
        if f and f ~= "" then
            table.insert(files, f)
        end
    end
    handle:close()

    if #files == 0 then
        mp.osd_message("load-all-media: no files returned", 3)
        return
    end

    -- Clear playlist and load first file, then append the rest
    mp.commandv("loadfile", files[1], "replace")
    for i = 2, #files do
        mp.commandv("loadfile", files[i], "append")
    end

    mp.osd_message(string.format("Loaded %d files", #files), 3)
end)
