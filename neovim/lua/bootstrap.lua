-- DEP
--
-- automatically install `chiyadev/dep` on startup
local path = vim.fn.stdpath("data") .. "/site/pack/deps/opt/dep"
if vim.fn.empty(vim.fn.glob(path)) > 0 then
	print("installing dep...")
  vim.fn.system({ "git", "clone", "--depth=1", "https://github.com/chiyadev/dep", path })
	print("dep installed.")
end
vim.cmd	'packadd dep'

-- side scrollbar with git support
require 'dep' {
	sync = "new",
	modules = {
		"dep.comments",
		"dep.motion",
		"dep.leader",
		"dep.themes",
	},
}
