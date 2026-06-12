[default, private]
main:
	@just --list

# Build the project (debug)
[group('build')]
build:
	cargo build

# Build release artifacts
[group('build')]
release:
	cargo build --release

# Run the test suite
[group('test')]
test:
	cargo test

# Run clippy with warnings as errors
[group('check')]
lint:
	cargo clippy --all-targets -- -D warnings

# Verify formatting without changing files
[group('check')]
fmt-check:
	cargo fmt --check

# Format the codebase
[group('check')]
fmt:
	cargo fmt

# Lint, format check, and tests
[group('check')]
check: lint fmt-check test

# Context is the parent directory so the sibling WCL/wisp path deps are
# available to the build.
# Build the official runtime container image (PRD §14)
[group('build')]
image tag='vmlab:latest':
	docker build -t {{tag}} -f Containerfile ..

# Open a viewer for each VM screen of a running lab ($VMLAB_VIEWER, default gvncviewer)
[group('lab')]
screens-open lab='mixed-lab':
	uv run python scripts/watch_screens.py --once --lab {{lab}}

# Watch for guest screens (labs and template builds) and open viewers as they appear
[group('lab')]
screens-watch:
	uv run python scripts/watch_screens.py

# Bring a lab up with each guest screen popping into a viewer as it boots
[group('lab')]
lab-up-watch dir='examples/mixed-lab': release
	cd {{dir}} && uv run python {{justfile_directory()}}/scripts/watch_screens.py -- {{justfile_directory()}}/target/release/vmlab up

# Run a template build with the build VM's screen visible (watch the installer)
[group('lab')]
template-build-watch dir='examples/templates/ubuntu-24.04': release
	cd {{dir}} && uv run python {{justfile_directory()}}/scripts/watch_screens.py -- {{justfile_directory()}}/target/release/vmlab template build
