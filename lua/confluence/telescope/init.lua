-- Telescope integration for Confluence
local has_telescope, pickers = pcall(require, 'telescope.pickers')
if not has_telescope then
  error('confluence-nvim requires telescope.nvim to be installed')
  return {}
end

local finders = require('telescope.finders')
local conf = require('telescope.config').values
local actions = require('telescope.actions')
local action_state = require('telescope.actions.state')
local api = require('confluence.api')

local M = {}

--- Open space browser picker
function M.spaces(opts)
  opts = opts or {}

  -- Fetch spaces from API
  api.list_spaces(function(err, spaces)
    if err then
      vim.notify('Failed to fetch spaces: ' .. err, vim.log.levels.ERROR)
      return
    end

    if #spaces == 0 then
      vim.notify('No spaces found', vim.log.levels.WARN)
      return
    end

    pickers
      .new(opts, {
        prompt_title = 'Confluence Spaces',
        finder = finders.new_table({
          results = spaces,
          entry_maker = function(space)
            return {
              value = space,
              display = string.format('🏢 %-10s  %s', space.key, space.name),
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
  end)
end

--- Open page browser for a space
---@param opts table Options including space_key
function M.pages(opts)
  opts = opts or {}
  local space_key = opts.space_key

  if not space_key then
    vim.notify('space_key is required', vim.log.levels.ERROR)
    return
  end

  -- Fetch pages from API
  api.list_pages(space_key, function(err, pages)
    if err then
      vim.notify('Failed to fetch pages: ' .. err, vim.log.levels.ERROR)
      return
    end

    if #pages == 0 then
      vim.notify('No pages found in space ' .. space_key, vim.log.levels.WARN)
      return
    end

    pickers
      .new(opts, {
        prompt_title = string.format('%s: Pages', space_key),
        finder = finders.new_table({
          results = pages,
          entry_maker = function(page)
            return {
              value = page,
              display = string.format('📄 %s', page.title),
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
  end)
end

--- Search Confluence
---@param opts table Options including optional query
function M.search(opts)
  opts = opts or {}

  -- Prompt for search query
  vim.ui.input({ prompt = 'Search Confluence: ' }, function(query)
    if not query or query == '' then
      return
    end

    -- Search via API
    api.search(query, function(err, results)
      if err then
        vim.notify('Search failed: ' .. err, vim.log.levels.ERROR)
        return
      end

      if #results == 0 then
        vim.notify('No results found for: ' .. query, vim.log.levels.INFO)
        return
      end

      pickers
        .new(opts, {
          prompt_title = 'Search: ' .. query,
          finder = finders.new_table({
            results = results,
            entry_maker = function(result)
              local space_info = result.space and ('🏢 ' .. result.space.key .. ' ') or ''
              local excerpt = result.excerpt or ''
              excerpt = excerpt:gsub('<[^>]+>', ''):sub(1, 80) -- Strip HTML and truncate

              return {
                value = result,
                display = string.format('🔍 %s%s\n  %s', space_info, result.title, excerpt),
                ordinal = result.title .. ' ' .. excerpt,
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
    end)
  end)
end

--- Open recent pages
function M.recent(opts)
  opts = opts or {}

  -- For MVP, show message that this feature is not yet implemented
  vim.notify('Recent pages feature coming soon! Use :Telescope confluence spaces to browse.', vim.log.levels.INFO)
end

return M
