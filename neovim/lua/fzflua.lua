local M = {}

M.workspace_symbols = function()
	require 'fzf-lua'.fzf_exec({ "DocumentFile", "ExaTool" })
end

return M
