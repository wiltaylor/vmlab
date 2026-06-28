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

# Context is the parent directory so the sibling WCL/wscript path deps are
# available to the build.
# Build the official runtime container image (PRD §14)
[group('build')]
image tag='vmlab:latest':
	docker build -t {{tag}} -f Containerfile ..

# Install the vmlab binary into the user profile (~/.cargo/bin)
[group('build')]
install:
	cargo install --path . --locked

# Bring a lab up (a VNC viewer opens per VM when the lab sets `gui = true`)
[group('lab')]
lab-up dir='examples/mixed-lab': release
	cd {{dir}} && {{justfile_directory()}}/target/release/vmlab up

# Stop a running lab gracefully (clones retained)
[group('lab')]
lab-down dir='examples/mixed-lab': release
	cd {{dir}} && {{justfile_directory()}}/target/release/vmlab down

# Tear a lab down completely: stop + delete clones and lab-local state
[group('lab')]
lab-destroy dir='examples/mixed-lab': release
	cd {{dir}} && {{justfile_directory()}}/target/release/vmlab destroy

# Launch the winsrv-desktop example (opens the WS2025 guest window)
[group('lab')]
winsrv-desktop: (lab-up 'examples/winsrv-desktop')

# The website + vmlab wskill are authored in wdoc and rendered by the `wcl` CLI.
# Install it from https://wcl.dev (or `cargo install --git …/wcl wcl`).

# Validate the vmlab wskill model and both projection templates
[group('docs')]
wskill-check:
	wcl check docs/wskills/vmlab/wskill.wcl
	wcl check docs/wskills/vmlab/wdoc/book/main.wcl
	wcl check docs/wskills/vmlab/wdoc/skill/main.wcl

# Build the documentation website to docs/_site (landing pages + embedded reference book)
[group('docs')]
docs-build: wskill-check
	wcl wdoc build docs/main.wcl --out docs/_site

# Serve the website locally with live reload; pass `true` to enable comment review mode (`just docs-serve true`)
[group('docs')]
docs-serve comment="false":
	wcl wdoc serve docs/main.wcl {{ if comment == "true" { "--comment" } else { "" } }}

# Regenerate the Claude Code skill at .claude/skills/vmlab from the wskill (single source)
[group('docs')]
skill-build: wskill-check
	wcl wdoc skill docs/wskills/vmlab/wdoc/skill/main.wcl --out .claude/skills/vmlab

# Remove generated site + wskill projections
[group('docs')]
docs-clean:
	rm -rf docs/_site docs/wskills/vmlab/out
