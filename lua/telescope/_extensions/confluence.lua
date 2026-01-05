-- Telescope extension registration
return require('telescope').register_extension({
  setup = function(ext_config, config)
    -- Extension-specific config can go here
  end,
  exports = {
    spaces = require('confluence.telescope').spaces,
    pages = require('confluence.telescope').pages,
    search = require('confluence.telescope').search,
    recent = require('confluence.telescope').recent,
  },
})
