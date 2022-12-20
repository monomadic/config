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
		require 'plugins.comments',
		require 'plugins.telescope',
		require 'plugins.themes', -- colorschemes
		require 'plugins.scrollbar',
		require 'plugins.fzf', -- fuzzy finder
		require 'plugins.menu', -- whichkey
		require 'plugins.lspsaga', -- better lsp ui
		require 'plugins.treesitter',
		require 'plugins.genghis', -- convenience file operations (new, rename, etc)
	}

	use {
		require 'lsp.mason',
		require 'lsp.rust',
		require 'lsp.lua',
		require 'lsp.null', -- null-ls
	}

	-- use 'andymass/vim-matchup'
	-- 	-- async formatting
	-- -- https://github.com/lukas-reineke/lsp-format.nvim
	-- use { 'lukas-reineke/lsp-format.nvim', config = function()
	-- 	require("lsp-format").setup()
	-- end }

	-- inline colors
	use { 'norcalli/nvim-colorizer.lua', config = function()
		require("colorizer").setup()
	end }

	use { 'ggandor/leap.nvim', config = function()
		require('leap').set_default_keymaps()
	end }
	-- git status in git gutter
	use { "lewis6991/gitsigns.nvim", requires = { "nvim-lua/plenary.nvim" }, config = function()
		require('gitsigns').setup {
			on_attach = function()
				local gs = package.loaded.gitsigns
				-- jump between git hunks
				vim.keymap.set('n', ']g', function()
					if vim.wo.diff then return ']g' end
					vim.schedule(function() gs.next_hunk() end)
					return '<Ignore>'
				end)
				vim.keymap.set('n', '[g', function()
					if vim.wo.diff then return '[g' end
					vim.schedule(function() gs.prev_hunk() end)
					return '<Ignore>'
				end)
				vim.api.nvim_set_hl(0, "GitSignsAdd", { fg = "#44FF00" })
				vim.api.nvim_set_hl(0, "GitSignsChange", { fg = "#FFFF00" })
				vim.api.nvim_set_hl(0, "GitSignsDelete", { fg = "#FF0088" })
			end
		}
	end }

	-- show lsp progress
	use {
		'j-hui/fidget.nvim',
		config = function() require("fidget").setup {} end
	}

	-- cargo
	use {
		"saecki/crates.nvim",
		event = { "BufRead Cargo.toml" },
		requires = { "nvim-lua/plenary.nvim" },
		config = function()
			require('crates').setup()
		end
	}

	-- async formatting
	-- https://github.com/lukas-reineke/lsp-format.nvim
	use { 'lukas-reineke/lsp-format.nvim', config = function()
		require("lsp-format").setup()
	end }
end)
