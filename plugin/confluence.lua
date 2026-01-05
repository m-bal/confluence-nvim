-- Confluence Neovim Plugin Registration
-- This file is automatically loaded by Neovim

-- Only load plugin once
if vim.g.loaded_confluence then
  return
end
vim.g.loaded_confluence = 1

-- Ensure Neovim version is compatible
if vim.fn.has('nvim-0.10') == 0 then
  vim.api.nvim_err_writeln('confluence.nvim requires Neovim >= 0.10.0')
  return
end

-- Plugin will be initialized when user calls require('confluence').setup()
