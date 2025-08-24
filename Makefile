# Claude Hook Advisor Makefile
# Similar to hashtag-search structure

.PHONY: build install clean test release help

# Default target
all: build

# Build the project in debug mode
build:
	cargo build

# Build in release mode for production
release:
	cargo build --release

# Install the binary using cargo (like hashtag-search)
install:
	cargo install --path .

# Install to ~/.local/bin manually
install-local: release
	mkdir -p ~/.local/bin
	cp target/release/claude-hook-advisor ~/.local/bin/
	@echo "claude-hook-advisor installed to ~/.local/bin/claude-hook-advisor"
	@echo "Make sure ~/.local/bin is in your PATH"

# Install system-wide (requires sudo)
install-system: release
	sudo cp target/release/claude-hook-advisor /usr/local/bin/
	@echo "claude-hook-advisor installed to /usr/local/bin/claude-hook-advisor"

# Run tests (single-threaded to avoid race conditions in temp directory tests)
test:
	cargo test -- --test-threads=1

# Clean build artifacts
clean:
	cargo clean

# Format code
fmt:
	cargo fmt

# Run clippy for linting
lint:
	cargo clippy -- -D warnings

# Check code without building
check:
	cargo check

# Run with example config
run-example:
	echo '{"session_id":"test","transcript_path":"","cwd":"","hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{"command":"npm install","description":"Install packages"}}' | cargo run -- --hook


# Show help
help:
	@echo "Available targets:"
	@echo "  build         - Build in debug mode"
	@echo "  release       - Build in release mode"
	@echo "  install       - Install using cargo (globally available)"
	@echo "  install-local - Install to ~/.local/bin"
	@echo "  install-system- Install system-wide (requires sudo)"
	@echo "  test          - Run tests"
	@echo "  clean         - Clean build artifacts"
	@echo "  fmt           - Format code"
	@echo "  lint          - Run clippy linting"
	@echo "  check         - Check code without building"
	@echo "  run-example   - Test with example JSON input"
	@echo "  help          - Show this help"