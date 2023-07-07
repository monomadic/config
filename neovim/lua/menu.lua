-- Right-click context menu

local M = {}

M.clear = function(menu)
	vim.cmd.aunmenu { 'PopUp' }
end

vim.cmd.aunmenu { 'PopUp' }
vim.cmd.nmenu('PopUp.Rename', ':lua vim.lsp.buf.rename()<CR>')
vim.cmd.nmenu('PopUp.Definition', ':lua vim.lsp.buf.definition()<CR>')
vim.cmd.nmenu('PopUp.Declaration', ':lua vim.lsp.buf.declaration()<CR>')
vim.cmd.nmenu('PopUp.TypeDefinition', ':lua vim.lsp.buf.type_definition()<CR>')
vim.cmd.nmenu('PopUp.Implementation', ':lua vim.lsp.buf.implementation()<CR>')
vim.cmd.nmenu('PopUp.References', ':lua vim.lsp.buf.references()<CR>')
vim.cmd.nmenu('PopUp.Inspect', ':Inspect<CR>')
vim.cmd.nmenu('PopUp.CodeActions', ':lua vim.lsp.buf.code_action()<CR>')

return M
