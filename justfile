[default, private]
main:
	@just --list

[group('build')]
# Build the project (debug)
build:
	cargo build

[group('build')]
# Build release artifacts
release:
	cargo build --release

[group('test')]
# Run the test suite
test:
	cargo test

[group('check')]
# Run clippy with warnings as errors
lint:
	cargo clippy --all-targets -- -D warnings

[group('check')]
# Verify formatting without changing files
fmt-check:
	cargo fmt --check

[group('check')]
# Format the codebase
fmt:
	cargo fmt

[group('check')]
# Lint, format check, and tests
check: lint fmt-check test
