-- Telescope integration for Confluence
local pickers = require('telescope.pickers')
local finders = require('telescope.finders')
local conf = require('telescope.config').values
local actions = require('telescope.actions')
local action_state = require('telescope.actions.state')

local M = {}

--- Open space browser picker
function M.spaces(opts)
  opts = opts or {}

  -- TODO: Fetch spaces from Rust API
  local spaces = {
    { key = 'PROJ', name = 'Project Documentation', page_count = 156 },
    { key = 'ENG', name = 'Engineering Docs', page_count = 89 },
    { key = 'DESIGN', name = 'Design System', page_count = 42 },
    { key = 'KB', name = 'Knowledge Base', page_count = 234 },
  }

  pickers
    .new(opts, {
      prompt_title = 'Confluence Spaces',
      finder = finders.new_table({
        results = spaces,
        entry_maker = function(space)
          return {
            value = space,
            display = string.format('[S] %-10s  %-40s %3d pages', space.key, space.name, space.page_count),
            ordinal = space.key .. ' ' .. space.name,
          }
        end,
      }),
      sorter = conf.generic_sorter(opts),
      attach_mappings = function(prompt_bufnr, map)
        actions.select_default:replace(function()
          local selection = action_state.get_selected_entry()
          actions.close(prompt_bufnr)

          -- Open pages for this space
          M.pages({ space_key = selection.value.key })
        end)

        -- Search within space
        map('i', '<C-s>', function()
          local selection = action_state.get_selected_entry()
          actions.close(prompt_bufnr)
          M.search({ space_key = selection.value.key })
        end)

        return true
      end,
    })
    :find()
end

--- Open page browser for a space
---@param opts table Options including space_key
function M.pages(opts)
  opts = opts or {}
  local space_key = opts.space_key or 'PROJ'

  -- TODO: Fetch pages from Rust API
  local pages = {
    { id = '123456', title = 'Getting Started Guide', updated = '2h ago', has_children = false },
    { id = '123457', title = 'Architecture', updated = '1d ago', has_children = true, child_count = 12 },
    { id = '123458', title = 'Development', updated = '3d ago', has_children = true, child_count = 8 },
    { id = '123459', title = 'Contributing Guidelines', updated = '1w ago', has_children = false },
  }

  pickers
    .new(opts, {
      prompt_title = string.format('%s: Pages', space_key),
      finder = finders.new_table({
        results = pages,
        entry_maker = function(page)
          local icon = page.has_children and '[F]' or '[P]'
          local child_info = page.has_children and string.format(' (%d children)', page.child_count) or ''

          return {
            value = page,
            display = string.format('%s %-50s  %s%s', icon, page.title, page.updated, child_info),
            ordinal = page.title,
          }
        end,
      }),
      sorter = conf.generic_sorter(opts),
      attach_mappings = function(prompt_bufnr, map)
        actions.select_default:replace(function()
          local selection = action_state.get_selected_entry()
          actions.close(prompt_bufnr)

          -- Open the page
          require('confluence').open_page(selection.value.id)
        end)

        -- Open in split
        map('i', '<C-x>', function()
          local selection = action_state.get_selected_entry()
          actions.close(prompt_bufnr)
          vim.cmd('split')
          require('confluence').open_page(selection.value.id)
        end)

        -- Open in vsplit
        map('i', '<C-v>', function()
          local selection = action_state.get_selected_entry()
          actions.close(prompt_bufnr)
          vim.cmd('vsplit')
          require('confluence').open_page(selection.value.id)
        end)

        return true
      end,
    })
    :find()
end

--- Search Confluence
---@param opts table Options including optional space_key
function M.search(opts)
  opts = opts or {}
  local query = opts.query or ''

  pickers
    .new(opts, {
      prompt_title = 'Search Confluence',
      finder = finders.new_dynamic({
        fn = function(prompt)
          if prompt == '' then
            return {}
          end

          -- TODO: Call Rust search API
          local results = {
            {
              id = '123456',
              title = '[PROJ] API Authentication Guide',
              excerpt = 'Overview of OAuth2 and JWT authentication methods...',
              space_key = 'PROJ',
            },
            {
              id = '123457',
              title = '[ENG] Auth Service Architecture',
              excerpt = 'The authentication service handles user login and...',
              space_key = 'ENG',
            },
          }

          return results
        end,
        entry_maker = function(result)
          return {
            value = result,
            display = string.format('%s\n  %s', result.title, result.excerpt),
            ordinal = result.title .. ' ' .. result.excerpt,
          }
        end,
      }),
      sorter = conf.generic_sorter(opts),
      attach_mappings = function(prompt_bufnr, map)
        actions.select_default:replace(function()
          local selection = action_state.get_selected_entry()
          actions.close(prompt_bufnr)
          require('confluence').open_page(selection.value.id)
        end)

        return true
      end,
    })
    :find()
end

--- Open recent pages
function M.recent(opts)
  opts = opts or {}

  -- TODO: Load from history file
  local recent = {
    { id = '123456', title = '[PROJ] API Authentication Guide', viewed = '10m ago', space_key = 'PROJ' },
    { id = '123457', title = '[ENG] Database Schema', viewed = '1h ago', space_key = 'ENG' },
    { id = '123458', title = '[PROD] Q1 Product Roadmap', viewed = '3h ago', space_key = 'PROD' },
  }

  pickers
    .new(opts, {
      prompt_title = 'Recent Confluence Pages',
      finder = finders.new_table({
        results = recent,
        entry_maker = function(page)
          return {
            value = page,
            display = string.format('%-60s  %s', page.title, page.viewed),
            ordinal = page.title,
          }
        end,
      }),
      sorter = conf.generic_sorter(opts),
      attach_mappings = function(prompt_bufnr, map)
        actions.select_default:replace(function()
          local selection = action_state.get_selected_entry()
          actions.close(prompt_bufnr)
          require('confluence').open_page(selection.value.id)
        end)

        return true
      end,
    })
    :find()
end

return M
