function HighlightMatches(query)
  local bufnr = vim.api.nvim_get_current_buf()
  local lang = require('nvim-treesitter.parsers').get_buf_lang(bufnr)
  local parser = vim.treesitter.get_parser(bufnr, lang)

  if not parser then
    print("No parser found for language:", lang)
    return
  end

  local ts_query = vim.treesitter.query.parse(lang, query)
  local ns_id = vim.api.nvim_create_namespace("treesitter_highlight_matches")

  -- Clear previous highlights
  vim.api.nvim_buf_clear_namespace(bufnr, ns_id, 0, -1)

  local tree = parser:parse()[1]
  local iter = ts_query:iter_captures(tree:root(), bufnr, 0, -1)

  for capture, node in iter do
    local start_row, start_col, end_row, end_col = node:range()
    vim.api.nvim_buf_add_highlight(
      bufnr,
      ns_id,
      "MatchWord",
      start_row,
      start_col,
      end_col
    )
  end
end
