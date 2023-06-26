-- local function custom_codeAction_callback(_, _, action)
-- 	print("attaching custom...")
-- 	print(vim.inspect(action))
-- end
--
-- vim.lsp.handlers['textDocument/codeAction'] = custom_codeAction_callback
--

    -- vim.api.nvim_create_autocmd('LspAttach', {
    --   callback = function(args)
    --     local client = vim.lsp.get_client_by_id(args.data.client_id)
    --     if client.server_capabilities.hoverProvider then
    --       vim.keymap.set('n', 'K', vim.lsp.buf.hover, { buffer = args.buf })
    --     end
    --   end,
    -- })
    --

		local current_symbol_index = 1

local function jump_to_next_symbol()
    local params = { textDocument = vim.lsp.util.make_text_document_params() }

    vim.lsp.buf_request(0, 'textDocument/documentSymbol', params, function(_, _, result)
        if not result or vim.tbl_isempty(result) then
            print("No symbols found")
            return
        end

        -- Check if result is a hierarchy of symbols or a flat list
        local symbols = result
        if result[1].children then
            symbols = {}
            for _, sym_info in ipairs(result) do
                vim.list_extend(symbols, sym_info.children)
            end
        end

        -- Jump to the location of the next symbol
        if current_symbol_index > #symbols then
            current_symbol_index = 1
        end
        local symbol = symbols[current_symbol_index]
        vim.lsp.util.jump_to_location({
            uri = symbol.location and symbol.location.uri or symbol.uri,
            range = symbol.location and symbol.location.range or symbol.range
        })

        current_symbol_index = current_symbol_index + 1
    end)
end

-- Bind the function to a key
vim.api.nvim_set_keymap('n', '<leader>j', '<cmd>lua jump_to_next_symbol()<CR>', {noremap = true, silent = true})
