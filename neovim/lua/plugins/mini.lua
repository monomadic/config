local key = Utils.key

return {
	'echasnovski/mini.nvim',
	version = false, -- false=main, '*'=stable
	config = function ()
		-- buffer removing
		-- MiniBufremove.unshow()
		require('mini.bufremove').setup()
		key('n', "<leader>Bd", MiniBufremove.delete, "delete")
		key('n', "<leader>Bw", MiniBufremove.wipeout, "wipeout")

		-- jump
		require('mini.jump').setup {}
	end
}
