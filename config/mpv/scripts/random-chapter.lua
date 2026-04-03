local mp = require "mp"

math.randomseed(os.time())

local function jump_to_random_chapter()
	local chapter_count = mp.get_property_number("chapter-list/count", 0)
	if chapter_count > 1 then
		local random_chapter = math.random(0, chapter_count - 1)
		mp.set_property_number("chapter", random_chapter)
	end
end

mp.register_script_message("play-random-chapter", jump_to_random_chapter)
