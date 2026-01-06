# Tests

Unit and integration tests for confluence-nvim.

## Running Tests

### Prerequisites

- Neovim >= 0.9.0
- [plenary.nvim](https://github.com/nvim-lua/plenary.nvim) (automatically installed by test runner)

### Run All Tests

```bash
cd tests
./run_tests.sh
```

### Run Specific Test File

```bash
nvim --headless --noplugin -u tests/minimal_init.lua \
  -c "PlenaryBustedFile tests/renderer_spec.lua"
```

## Test Structure

- `renderer_spec.lua` - Unit tests for the HTML renderer
- `rendering_spec.lua` - Integration tests for page rendering (placeholders)
- `minimal_init.lua` - Minimal Neovim config for testing
- `run_tests.sh` - Test runner script

## Writing Tests

Tests use [plenary.nvim](https://github.com/nvim-lua/plenary.nvim)'s busted-style testing framework.

Example:
```lua
describe('my feature', function()
  it('should do something', function()
    assert.equals('expected', 'actual')
  end)
end)
```
