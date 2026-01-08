# Build configuration for cross-compilation

# Default recipe
default:
    just --list

# Run in development mode
run:
    cargo run

# Run in release mode
run-release:
    cargo run --release

# Build for current platform
build:
    cargo build --release

# Build for macOS (from macOS)
build-macos:
    cargo build --release --target x86_64-apple-darwin
    cargo build --release --target aarch64-apple-darwin

# Build for Windows (from macOS using cross)
build-windows:
    cargo install cross
    cross build --release --target x86_64-pc-windows-gnu

# Test all crates
test:
    cargo test --workspace

# Clean build artifacts
clean:
    cargo clean

# Format code
fmt:
    cargo fmt --all

# Run clippy
clippy:
    cargo clippy --workspace -- -D warnings
