--
-- LSP
--
vim.api.nvim_create_autocmd('LspAttach', {
	callback = function(args)
		if not (args.data and args.data.client_id) then
			return
		end

		-- local bufnr = args.buf
		local client = vim.lsp.get_client_by_id(args.data.client_id)
		local keymap = vim.keymap.set

		-- print(vim.inspect(client.server_capabilities))

		-- hoverProvider
		if client.server_capabilities.hoverProvider then
			vim.keymap.set('n', 'K', vim.lsp.buf.hover, { buffer = args.buf })

			vim.lsp.handlers["textDocument/hover"] = vim.lsp.with(
				vim.lsp.handlers.hover, {
					-- Use a sharp border with `FloatBorder` highlights
					border = "single",
					-- add the title in hover float window
					-- title = "hover"
				}
			)
		end

		-- codeActionProvider
		if client.server_capabilities.codeActionProvider then
			vim.keymap.set('n', '<leader>La', vim.lsp.buf.code_action, { desc = "code action" })
		end

		vim.keymap.set('n', ']d', vim.diagnostic.goto_next, { desc = "Next diagnostic" })
		vim.keymap.set('n', '[d', vim.diagnostic.goto_prev, { desc = "Previous diagnostic" })
		vim.keymap.set('n', ']e', function()
			vim.diagnostic.goto_next {
				severity = vim.diagnostic.severity.ERROR
			}
		end, { desc = "Next error" })
		vim.keymap.set('n', '[e', function()
			vim.diagnostic.goto_prev {
				severity = vim.diagnostic.severity.ERROR
			}
		end, { desc = "Next error" })

		keymap('n', '<leader>Lr', vim.lsp.buf.references, { desc = "references" })
		keymap('n', '<leader>Li', vim.lsp.buf.implementation, { desc = "implementation" })
		keymap('n', "<leader>Ld", vim.lsp.buf.definition, { desc = "definition" })
		keymap('n', "<leader>LD", vim.lsp.buf.declaration, { desc = "declaration" })
		keymap('n', '<leader>Ls', vim.lsp.buf.workspace_symbol, { desc = "workspace symbol" })

		keymap('n', "gd", vim.lsp.buf.definition, { desc = "definition" })
		keymap('n', "<Enter>", vim.lsp.buf.definition, { desc = "definition" })
		keymap('n', "gc", vim.lsp.buf.declaration, { desc = "declaration" })
	end,
})

local current_symbol_index = 1

local function flatten_symbols(symbols, result)
	result = result or {}
	for _, symbol in ipairs(symbols) do
		table.insert(result, symbol)
		if symbol.children then
			flatten_symbols(symbol.children, result)
		end
	end
	return result
end

local function jump_to_next_symbol()
	local params = { textDocument = vim.lsp.util.make_text_document_params() }

	vim.lsp.buf_request(0, 'textDocument/documentSymbol', params, function(_, _, result)
		if not result or vim.tbl_isempty(result) then
			print("No symbols found")
			return
		end

		local symbols = flatten_symbols(result)

		-- Reset index if out of bounds
		if current_symbol_index > #symbols or current_symbol_index < 1 then
			current_symbol_index = 1
		end

		-- Get the symbol at the current index
		local symbol = symbols[current_symbol_index]
		if not symbol then
			print("Could not find symbol")
			return
		end

		-- Get the range of the symbol
		local target_range = symbol.selectionRange or symbol.range

		-- Jump to the location of the symbol
		vim.lsp.util.jump_to_location({
			uri = symbol.location and symbol.location.uri or target_range.uri,
			range = target_range
		})

		-- Increment the index for the next jump
		current_symbol_index = current_symbol_index + 1
	end)
end

-- Bind the function to a key
vim.keymap.set('n', '<leader>J', jump_to_next_symbol, { noremap = true, silent = true })
