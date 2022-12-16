vim.cmd [[ command -range=% UpperCase '<,'>s/[a-zA-Z]/\U&/g ]]
vim.cmd [[ command -range=% LowerCase '<,'>s/[a-z]/\l&/g ]]
vim.cmd [[ command -range=% CamelCase '<,'>s/[ |-][a-zA-Z]/\U&/g ]]
vim.cmd [[ command -range=% Capitalize '<,'>s/[ |-][a-zA-Z]/\U&/g ]]
