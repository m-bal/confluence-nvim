.PHONY: test test-rust test-lua test-all coverage lint fmt clean dev

# Colors for output
BLUE := \033[0;34m
GREEN := \033[0;32m
RED := \033[0;31m
NC := \033[0m # No Color

# Run all tests
test: test-rust test-lua
	@echo "$(GREEN)✓ All tests passed$(NC)"

# Rust unit tests
test-rust:
	@echo "$(BLUE)Running Rust unit tests...$(NC)"
	@cargo test --lib

# Lua integration tests
test-lua:
	@echo "$(BLUE)Running Lua integration tests...$(NC)"
	@nvim --headless -c "PlenaryBustedDirectory tests/integration/ {minimal_init = 'tests/minimal_init.lua'}"

# All tests including visual
test-all: test
	@echo "$(BLUE)Running visual regression tests...$(NC)"
	@nvim --headless -c "PlenaryBustedDirectory tests/visual/ {minimal_init = 'tests/minimal_init.lua'}"

# Coverage report
coverage:
	@echo "$(BLUE)Generating coverage report...$(NC)"
	@cargo tarpaulin --out Html --output-dir coverage
	@echo "$(GREEN)✓ Coverage report generated at coverage/index.html$(NC)"

# Linting
lint:
	@echo "$(BLUE)Running linters...$(NC)"
	@cargo clippy -- -D warnings
	@echo "$(GREEN)✓ Linting passed$(NC)"

# Formatting
fmt:
	@echo "$(BLUE)Formatting code...$(NC)"
	@cargo fmt
	@stylua lua/ tests/ || true
	@echo "$(GREEN)✓ Code formatted$(NC)"

# Clean build artifacts
clean:
	@echo "$(BLUE)Cleaning build artifacts...$(NC)"
	@cargo clean
	@rm -rf target/
	@echo "$(GREEN)✓ Clean complete$(NC)"

# Development mode - watch and rerun tests
dev:
	@echo "$(BLUE)Starting development mode (auto-rerun tests)...$(NC)"
	@cargo watch -x test

# Start mock server for manual testing
mock-server:
	@echo "$(BLUE)Starting mock Confluence server on port 8080...$(NC)"
	@python3 tests/mock_server.py

# Build release version
build:
	@echo "$(BLUE)Building release version...$(NC)"
	@cargo build --release
	@echo "$(GREEN)✓ Build complete$(NC)"

# Install for local testing
install: build
	@echo "$(BLUE)Installing plugin locally...$(NC)"
	@mkdir -p ~/.local/share/nvim/site/pack/confluence/start/confluence-nvim
	@cp -r lua plugin ~/.local/share/nvim/site/pack/confluence/start/confluence-nvim/
	@cp target/release/libconfluence_nvim.so ~/.local/share/nvim/site/pack/confluence/start/confluence-nvim/ || true
	@echo "$(GREEN)✓ Plugin installed$(NC)"

# Help
help:
	@echo "Confluence Neovim Plugin - Make targets:"
	@echo ""
	@echo "  $(GREEN)make test$(NC)          - Run all tests (Rust + Lua)"
	@echo "  $(GREEN)make test-rust$(NC)     - Run only Rust unit tests"
	@echo "  $(GREEN)make test-lua$(NC)      - Run only Lua integration tests"
	@echo "  $(GREEN)make test-all$(NC)      - Run all tests including visual"
	@echo "  $(GREEN)make coverage$(NC)      - Generate coverage report"
	@echo "  $(GREEN)make lint$(NC)          - Run linters (clippy)"
	@echo "  $(GREEN)make fmt$(NC)           - Format code (rustfmt + stylua)"
	@echo "  $(GREEN)make clean$(NC)         - Clean build artifacts"
	@echo "  $(GREEN)make dev$(NC)           - Watch mode (auto-rerun tests)"
	@echo "  $(GREEN)make mock-server$(NC)   - Start mock Confluence API server"
	@echo "  $(GREEN)make build$(NC)         - Build release version"
	@echo "  $(GREEN)make install$(NC)       - Install plugin locally"
	@echo "  $(GREEN)make help$(NC)          - Show this help message"
