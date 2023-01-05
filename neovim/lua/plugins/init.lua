-- PLUGINS
--
--   PackerCompile: compile plugins
--   PackerClean: remove unused plugs
--   PackerInstall: add new plugins
--   PackerUpdate: PackerClean, PackerUpdate, PackerInstall
--   PackerSync: PackerUpdate, PackerCompile
--
--  TODO:
--  - https://github.com/MunifTanjim/nui.nvim
--	- https://github.com/folke/lazy.nvim
--

-- vim.defer_fn(function()
-- 	pcall(require, "impatient")
-- end, 0)

print('bootstrapping plugins')

-- add binaries installed by mason.nvim to path
-- local is_windows = vim.loop.os_uname().sysname == "Windows_NT"
-- vim.env.PATH = vim.env.PATH .. (is_windows and ";" or ":") .. vim.fn.stdpath "data" .. "/mason/bin"

-- autoinstall packer:
local packer_exists = pcall(require, "packer")
if not packer_exists then
	local install_path = vim.fn.stdpath("data") .. "/site/pack/packer/start/packer.nvim"
	if vim.fn.empty(vim.fn.glob(install_path)) > 0 then
		print("downloading packer...")
		vim.fn.system({ "git", "clone", "--depth", "1", "https://github.com/wbthomason/packer.nvim", install_path })
		-- vim.cmd 'packadd packer.nvim'
	end
end

-- note: on treesitter error, ensure :TSInstall vim
vim.cmd "packadd packer.nvim"

-- install binaries from mason.nvim & tsparsers
vim.api.nvim_create_autocmd("User", {
	pattern = "PackerComplete",
	callback = function()
		vim.cmd "bw | silent! MasonInstallAll" -- close packer window
		require("packer").loader "nvim-treesitter"
	end,
})

local packer = require 'packer'

packer.init({
	auto_clean = true,
	compile_on_sync = true,
	git = { clone_timeout = 6000 },
	display = {
		working_sym = "ﲊ",
		error_sym = "✗ ",
		done_sym = " ",
		removed_sym = " ",
		moved_sym = "",
		open_fn = function()
			return require("packer.util").float { border = "single" }
		end,
	},
})

packer.startup(function(use)
	-- speed up lua modules
	use { 'lewis6991/impatient.nvim', config = function()
		require('impatient')
	end }

	use {
		'wbthomason/packer.nvim', -- packer package manager
		require 'plugins.transparent', -- background transparency
		require 'plugins.comments',
		require 'plugins.telescope',
		require 'plugins.themes', -- colorschemes
		require 'plugins.color-highlight', -- highlight colors
		require 'plugins.scrollbar',
		require 'plugins.fzf', -- fuzzy finder
		--require 'plugins.menu', -- whichkey
		require 'plugins.lspsaga', -- better lsp ui
		require 'plugins.treesitter',
		require 'plugins.genghis', -- convenience file operations (new, rename, etc)
		require 'plugins.wiki', -- vimwiki
		require 'plugins.reading-mode', -- flowstate, zen modes
		require 'plugins.autocomplete',
		require 'plugins.todo', -- todo comments
		require 'plugins.git',
		require 'plugins.drex',
		-- require 'plugins.notifications',
	}

	use {
		require 'lsp.mason',
		require 'lsp.rust',
		require 'lsp.lua',
		require 'lsp.null', -- null-ls
		require 'lsp.progress', -- fidget (show lsp process status)
		require 'lsp.format', -- format using lsp
		require 'lsp.cargo', -- rust cargo support
		require 'lsp.glance', -- lsp navigation
		require 'lsp.typescript',
		require 'lsp.inlay-hints',
		require 'lsp.signature', -- function signatures
	}

	-- use 'andymass/vim-matchup'

	-- improved search feedback
	-- use { 'kevinhwang91/nvim-hlslens' }
	use {
		"kevinhwang91/nvim-hlslens",
		requires = { "petertriho/nvim-scrollbar" },
		config = function()
			-- require('hlslens').setup() is not required
			require("scrollbar.handlers.search").setup({
				-- hlslens config overrides
			})
		end,
	}

	use { 'ggandor/leap.nvim', config = function()
		require('leap').set_default_keymaps()
	end }

end)
