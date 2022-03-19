local fzf = require("fzf")

coroutine.wrap(function()
  local result = fzf.fzf({"choice 1", "choice 2"}, "--ansi")
  -- result is a list of lines that fzf returns, if the user has chosen
  if result then
    print(result[1])
  end
end)()
