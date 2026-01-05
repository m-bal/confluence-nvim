-- Minimal init for running tests
-- This sets up only what's needed for testing

-- Add current directory to package path
local root = vim.fn.fnamemodify(vim.fn.getcwd(), ':p')
vim.opt.runtimepath:append(root)

-- Add plenary to runtime path
local plenary_path = vim.fn.stdpath('data') .. '/site/pack/vendor/start/plenary.nvim'
if vim.fn.isdirectory(plenary_path) == 0 then
  vim.fn.system({
    'git',
    'clone',
    'https://github.com/nvim-lua/plenary.nvim',
    plenary_path,
  })
end
vim.opt.runtimepath:append(plenary_path)

-- Disable swapfile and backup for tests
vim.opt.swapfile = false
vim.opt.backup = false

-- Set up test environment variables
vim.env.CONFLUENCE_TEST_MODE = '1'

-- Load the plugin
require('plenary.busted')
