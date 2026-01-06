-- Minimal init for testing

-- Add project root to package.path
local project_root = vim.fn.fnamemodify(vim.fn.getcwd(), ':p:h:h')
vim.opt.rtp:append(project_root)

-- Add plenary to runtime path
local plenary_path = vim.fn.expand('~/.local/share/nvim/site/pack/vendor/start/plenary.nvim')
vim.opt.rtp:append(plenary_path)

-- Require plenary
require('plenary.busted')
