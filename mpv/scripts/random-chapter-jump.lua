math.randomseed(os.time())

function random_jump()
	local playlist_count = mp.get_property_number("playlist-count", 0)
	if playlist_count == 0 then return end

	-- Select a random playlist index
	local random_index = math.random(0, playlist_count - 1)
	mp.set_property_number("playlist-pos", random_index)
end

function jump_to_random_chapter()
	local chapter_count = mp.get_property_number("chapter-list/count", 0)
	if chapter_count > 1 then
		local random_chapter = math.random(0, chapter_count - 1)
		mp.set_property_number("chapter", random_chapter)
	end
end

-- mp.add_key_binding("Ctrl+r", "random_jump", random_jump)
-- mp.register_event("file-loaded", jump_to_random_chapter)
