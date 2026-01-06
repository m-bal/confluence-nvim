-- Minimal init for testing

-- Get the directory where this file is located
local init_path = debug.getinfo(1, 'S').source:sub(2)
local test_dir = vim.fn.fnamemodify(init_path, ':h')
local project_root = vim.fn.fnamemodify(test_dir, ':h')

-- Add project root to runtime path
vim.opt.rtp:prepend(project_root)

-- Add plenary to runtime path
local plenary_path = vim.fn.expand('~/.local/share/nvim/site/pack/vendor/start/plenary.nvim')
if vim.fn.isdirectory(plenary_path) == 1 then
  vim.opt.rtp:prepend(plenary_path)
end

-- Ensure plugins are loaded
vim.cmd('runtime! plugin/**/*.lua')
vim.cmd('runtime! plugin/**/*.vim')
