# shellcheck disable=SC2148
# Justfile for causal-dynamical-triangulations development workflow
# Install just: https://github.com/casey/just
# Usage: just <command> or just --list

# Use bash with strict error handling for all recipes
set shell := ["bash", "-euo", "pipefail", "-c"]

# Internal helpers: ensure external tooling is installed
_ensure-actionlint:
    #!/usr/bin/env bash
    set -euo pipefail
    command -v actionlint >/dev/null || { echo "❌ 'actionlint' not found. See 'just setup' or https://github.com/rhysd/actionlint"; exit 1; }

_ensure-jq:
    #!/usr/bin/env bash
    set -euo pipefail
    command -v jq >/dev/null || { echo "❌ 'jq' not found. See 'just setup' or install: brew install jq"; exit 1; }

_ensure-git-cliff:
    #!/usr/bin/env bash
    set -euo pipefail
    command -v git-cliff >/dev/null || {
        echo "❌ 'git-cliff' not found. Install via Homebrew: brew install git-cliff"
        echo "   Or via Cargo: cargo install git-cliff"
        exit 1
    }

_ensure-npx:
    #!/usr/bin/env bash
    set -euo pipefail
    command -v npx >/dev/null || { echo "❌ 'npx' not found. See 'just setup' or install Node.js (for npx tools): https://nodejs.org"; exit 1; }

_ensure-dprint:
    #!/usr/bin/env bash
    set -euo pipefail
    command -v dprint >/dev/null || { echo "❌ 'dprint' not found. See 'just setup' or install: cargo install dprint"; exit 1; }

_ensure-shellcheck:
    #!/usr/bin/env bash
    set -euo pipefail
    command -v shellcheck >/dev/null || { echo "❌ 'shellcheck' not found. See 'just setup' or https://www.shellcheck.net"; exit 1; }

_ensure-shfmt:
    #!/usr/bin/env bash
    set -euo pipefail
    command -v shfmt >/dev/null || { echo "❌ 'shfmt' not found. See 'just setup' or install: brew install shfmt"; exit 1; }

# Internal helper: ensure taplo is installed
_ensure-taplo:
    #!/usr/bin/env bash
    set -euo pipefail
    command -v taplo >/dev/null || { echo "❌ 'taplo' not found. See 'just setup' or install: brew install taplo (or: cargo install taplo-cli)"; exit 1; }

# Internal helper: ensure typos-cli is installed
_ensure-typos:
    #!/usr/bin/env bash
    set -euo pipefail
    command -v typos >/dev/null || { echo "❌ 'typos' not found. See 'just setup' or install: cargo install typos-cli"; exit 1; }

# Internal helper: ensure uv is installed
_ensure-uv:
    #!/usr/bin/env bash
    set -euo pipefail
    command -v uv >/dev/null || { echo "❌ 'uv' not found. See 'just setup' or https://github.com/astral-sh/uv"; exit 1; }

_ensure-yamllint:
    #!/usr/bin/env bash
    set -euo pipefail
    command -v yamllint >/dev/null || { echo "❌ 'yamllint' not found. See 'just setup' or install: brew install yamllint (macOS), pip install yamllint, or uv tool install yamllint"; exit 1; }

# GitHub Actions workflow validation
action-lint: _ensure-actionlint
    #!/usr/bin/env bash
    set -euo pipefail
    files=()
    while IFS= read -r -d '' file; do
        files+=("$file")
    done < <(git ls-files -z '.github/workflows/*.yml' '.github/workflows/*.yaml')
    if [ "${#files[@]}" -gt 0 ]; then
        printf '%s\0' "${files[@]}" | xargs -0 actionlint
    else
        echo "No workflow files found to lint."
    fi

# Benchmarks
bench:
    cargo bench --workspace

# Compile benchmarks without running them, treating warnings as errors.
# This catches bench/release-profile-only warnings (e.g. debug_assertions-gated unused vars)
# that won't show up in normal debug-profile `cargo test` / `cargo clippy` runs.
bench-compile:
    RUSTFLAGS='-D warnings' cargo bench --workspace --no-run

# Build commands
build:
    cargo build

build-release:
    cargo build --release

# Changelog management
changelog: _ensure-uv _ensure-git-cliff
    uv run changelog-utils generate

changelog-tag version: _ensure-uv
    uv run changelog-utils tag {{version}}

changelog-update: changelog
    @echo "📝 Changelog updated successfully!"
    @echo "To create a git tag with changelog content for a specific version, run:"
    @echo "  just changelog-tag <version>  # e.g., just changelog-tag v0.4.2"

# Check (non-mutating): run all linters/validators
check: lint
    @echo "✅ Checks complete!"

# Fast compile check (no binary produced)
check-fast:
    cargo check

# CI simulation: comprehensive validation (matches .github/workflows/ci.yml)
# Runs: checks + all tests (Rust + Python) + examples + bench compile
ci: check bench-compile test-all
    @echo "🎯 CI checks complete!"

# CI with performance baseline
ci-baseline tag="ci":
    {{just_executable()}} ci
    {{just_executable()}} perf-baseline {{tag}}

# Clean build artifacts
clean:
    cargo clean
    rm -rf target/tarpaulin
    rm -rf coverage_report
    rm -rf coverage

# Code quality and formatting
clippy:
    cargo clippy --workspace --all-targets --all-features -- -D warnings -W clippy::pedantic -W clippy::nursery -W clippy::cargo

# Pre-commit workflow: comprehensive validation (checks + tests + release + benches + kani)
commit-check: check test-all test-release bench-compile kani-fast
    @echo "🚀 Ready to commit! All checks passed."

# Coverage analysis for local development (HTML output)
coverage:
    cargo tarpaulin --exclude-files 'benches/**' --exclude-files 'examples/**' --exclude-files 'tests/**' --out Html --output-dir target/tarpaulin
    @echo "📊 Coverage report generated: target/tarpaulin/tarpaulin-report.html"

# Coverage analysis for CI (XML output for codecov/codacy)
coverage-ci:
    cargo tarpaulin --exclude-files 'benches/**' --exclude-files 'examples/**' --exclude-files 'tests/**' --out Xml --output-dir coverage

coverage-report *args: _ensure-uv
    uv run coverage_report {{args}}

# Default recipe shows available commands
default:
    @just --list

doc-check:
    RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps --document-private-items

# Examples and validation
examples:
    ./scripts/run_all_examples.sh

# Fix (mutating): apply formatters/auto-fixes
fix: toml-fmt fmt python-fix shell-fmt markdown-fix
    @echo "✅ Fixes applied!"

fmt:
    cargo fmt --all

fmt-check:
    cargo fmt --all -- --check

# Help workflows
help-workflows:
    @echo "Common Just workflows:"
    @echo "  just fix               # Apply formatters/auto-fixes (mutating)"
    @echo "  just check             # Run lint/validators (non-mutating)"
    @echo "  just check-fast        # Fast compile check (cargo check)"
    @echo "  just ci                # Full CI run (checks + all tests + examples + bench compile)"
    @echo "  just ci-baseline       # CI + save performance baseline"
    @echo "  just commit-check      # Comprehensive pre-commit validation"
    @echo ""
    @echo "Testing:"
    @echo "  just test              # Lib and doc tests only (fast, used by CI)"
    @echo "  just test-integration  # Integration tests (tests/)"
    @echo "  just test-all          # All tests (lib + doc + integration + Python)"
    @echo "  just test-python       # Python tests only (pytest)"
    @echo "  just test-release      # All tests in release mode"
    @echo "  just test-cli          # CLI integration tests only"
    @echo "  just test-examples     # Run all examples"
    @echo "  just examples          # Run all example scripts"
    @echo "  just coverage          # Generate coverage report (HTML)"
    @echo "  just coverage-ci       # Generate coverage for CI (XML)"
    @echo ""
    @echo "Quality Check Groups:"
    @echo "  just lint          # All linting (code + docs + config)"
    @echo "  just lint-code     # Code linting (Rust, Python, Shell)"
    @echo "  just lint-docs     # Documentation linting (Markdown, Spelling)"
    @echo "  just lint-config   # Configuration validation (JSON, TOML, Actions)"
    @echo ""
    @echo "Formal Verification:"
    @echo "  just kani          # Run all Kani formal verification proofs"
    @echo "  just kani-fast     # Run fast Kani verification (ActionConfig only)"
    @echo ""
    @echo "Benchmark System:"
    @echo "  just bench              # Run all benchmarks"
    @echo "  just bench-compile      # Compile benchmarks without running"
    @echo ""
    @echo "Performance Analysis:"
    @echo "  just perf-help     # Show performance analysis commands"
    @echo "  just perf-check    # Check for performance regressions"
    @echo "  just perf-baseline # Save current performance as baseline"
    @echo ""
    @echo "Changelog:"
    @echo "  just changelog            # Generate/update CHANGELOG.md"
    @echo "  just changelog-tag <ver>  # Create git tag with changelog content"
    @echo ""
    @echo "Running:"
    @echo "  just run -- <args>  # Run with custom arguments"
    @echo "  just run-example    # Run with example arguments"
    @echo "  just run-simulation # Run basic_simulation.sh example script"
    @echo ""
    @echo "Note: Some recipes require external tools. Run 'just setup' for full environment setup."

# Kani formal verification
kani:
    cargo kani

kani-fast:
    cargo kani --harness verify_action_config

# All linting: code + documentation + configuration
lint: lint-code lint-docs lint-config

# Code linting: Rust (fmt-check, clippy, docs) + Python (ruff, ty, mypy) + Shell scripts
lint-code: fmt-check clippy doc-check python-lint shell-lint

# Configuration validation: JSON, TOML, YAML, GitHub Actions workflows
lint-config: validate-json toml-lint toml-fmt-check yaml-lint action-lint

# Documentation linting: Markdown + spell checking
lint-docs: markdown-check spell-check

markdown-check: _ensure-dprint
    dprint check

# Markdown and YAML: apply auto-fixes (mutating)
markdown-fix: _ensure-dprint
    dprint fmt

markdown-lint: markdown-check

perf-baseline tag="": _ensure-uv
    #!/usr/bin/env bash
    set -euo pipefail
    tag_value="{{tag}}"
    if [ -n "$tag_value" ]; then
        uv run performance-analysis --save-baseline --tag "$tag_value"
    else
        uv run performance-analysis --save-baseline
    fi

perf-check threshold="10.0": _ensure-uv
    uv run performance-analysis --threshold {{threshold}}

perf-compare file: _ensure-uv
    uv run performance-analysis --compare "{{file}}"

perf-help:
    @echo "Performance Analysis Commands:"
    @echo "  just perf-baseline [tag]    # Save current performance as baseline (optionally tagged)"
    @echo "  just perf-check [threshold] # Check for regressions (default: 10% threshold)"
    @echo "  just perf-report [file]     # Generate performance report"
    @echo "  just perf-trends [days]     # Analyze trends over N days (default: 7)"
    @echo "  just perf-compare <file>    # Compare with specific baseline file"
    @echo ""
    @echo "Examples:"
    @echo "  just perf-baseline v1.0.0   # Save tagged baseline"
    @echo "  just perf-check 5.0         # Check with 5% threshold"
    @echo "  just perf-report my_report.md # Save report to specific file"
    @echo "  just perf-trends 30         # Analyze last 30 days"

perf-report file="": _ensure-uv
    #!/usr/bin/env bash
    set -euo pipefail
    file_value="{{file}}"
    if [ -n "$file_value" ]; then
        uv run performance-analysis --report "$file_value"
    else
        timestamp=$(date +"%Y%m%d_%H%M%S")
        uv run performance-analysis --report "performance_report_${timestamp}.md"
        echo "📄 Report saved to: performance_report_${timestamp}.md"
    fi

perf-trends days="7": _ensure-uv
    uv run performance-analysis --trends {{days}}

python-check: _ensure-uv python-typecheck
    uv run ruff format --check scripts/
    uv run ruff check scripts/

# Python code quality
python-fix: _ensure-uv
    uv run ruff check scripts/ --fix
    uv run ruff format scripts/

python-lint: python-check

python-typecheck: _ensure-uv
    uv run ty check scripts/
    cd scripts && uv run mypy . --exclude tests

# Running the binary
run *args:
    cargo run --bin cdt {{args}}

run-example:
    cargo run --bin cdt -- -v 32 -t 3

run-release *args:
    cargo run --release --bin cdt {{args}}

# Run example simulation script
run-simulation:
    ./examples/scripts/basic_simulation.sh

# cspell:ignore oldname newname

# Development setup
setup:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "Setting up causal-dynamical-triangulations development environment..."
    echo "Note: Rust toolchain and components are managed by rust-toolchain.toml"
    echo ""
    echo "Installing Rust components..."
    rustup component add clippy rustfmt rust-docs rust-src
    echo ""
    echo "Additional tools (will check if installed):"
    for tool in uv actionlint shfmt shellcheck jq taplo yamllint dprint typos cargo-tarpaulin git-cliff; do
        if command -v "$tool" &> /dev/null; then
            echo "  ✓ $tool installed"
        else
            echo "  ✗ $tool NOT installed"
            case "$tool" in
                uv)
                    echo "    Install: https://github.com/astral-sh/uv"
                    echo "    macOS: brew install uv"
                    echo "    Linux/WSL: curl -LsSf https://astral.sh/uv/install.sh | sh"
                    ;;
                actionlint|yamllint)
                    echo "    macOS: brew install $tool"
                    echo "    pip: pip install $tool"
                    echo "    uv: uv tool install $tool"
                    ;;
                shfmt|shellcheck|jq|taplo)
                    echo "    macOS: brew install $tool"
                    if [ "$tool" = "taplo" ]; then
                        echo "    Or: cargo install taplo-cli"
                    fi
                    ;;
                dprint)
                    echo "    Install: cargo install dprint"
                    ;;
                typos)
                    echo "    Install: cargo install typos-cli"
                    ;;
                cargo-tarpaulin)
                    echo "    Install: cargo install cargo-tarpaulin"
                    ;;
                git-cliff)
                    echo "    macOS: brew install git-cliff"
                    echo "    Or: cargo install git-cliff"
                    ;;
            esac
        fi
    done
    echo ""
    if ! command -v uv &> /dev/null; then
        echo "❌ 'uv' is required but not installed. Please install it first (see instructions above)."
        exit 1
    fi
    echo ""
    echo "Installing Python tooling (ruff, mypy, pytest, and related dependencies)..."
    uv sync --group dev
    echo ""
    kani_version="0.66.0"
    echo "Ensuring Kani verifier (kani-verifier ${kani_version}) is installed..."
    installed_kani_version="$(cargo kani --version 2>/dev/null | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | head -1 || true)"
    if [[ "$installed_kani_version" == "$kani_version" ]]; then
        echo "  ✓ kani-verifier $installed_kani_version"
    else
        cargo install --locked --force --version "${kani_version}" kani-verifier
        cargo kani --version
    fi
    echo ""
    echo "Building project..."
    cargo build
    echo "✅ Setup complete! Run 'just help-workflows' to see available commands."

# Shell scripts: lint/check (non-mutating)
shell-check: _ensure-shellcheck _ensure-shfmt
    #!/usr/bin/env bash
    set -euo pipefail
    files=()
    while IFS= read -r -d '' file; do
        files+=("$file")
    done < <(git ls-files -z '*.sh')
    if [ "${#files[@]}" -gt 0 ]; then
        printf '%s\0' "${files[@]}" | xargs -0 -n4 shellcheck -x
        printf '%s\0' "${files[@]}" | xargs -0 shfmt -d
    else
        echo "No shell files found to check."
    fi

# Shell scripts: format (mutating)
shell-fmt: _ensure-shfmt
    #!/usr/bin/env bash
    set -euo pipefail
    files=()
    while IFS= read -r -d '' file; do
        files+=("$file")
    done < <(git ls-files -z '*.sh')
    if [ "${#files[@]}" -gt 0 ]; then
        echo "🧹 shfmt -w (${#files[@]} files)"
        printf '%s\0' "${files[@]}" | xargs -0 shfmt -w
    else
        echo "No shell files found to format."
    fi
    # Note: justfiles are not shell scripts and are excluded from shellcheck

shell-lint: shell-check

# Spell check (typos)
spell-check: _ensure-typos
    #!/usr/bin/env bash
    set -euo pipefail
    files=()
    # Use -z for NUL-delimited output to handle filenames with spaces.
    #
    # Note: For renames/copies, `git status --porcelain -z` emits *two* NUL-separated paths.
    # The field order is deterministic: the first path (filename) is the destination/new path
    # and the second path (other_path) is the source/old path. We prefer whichever exists on
    # disk (typically the destination) to avoid passing stale paths to typos.
    while IFS= read -r -d '' status_line; do
        status="${status_line:0:2}"
        filename="${status_line:3}"

        # For renames/copies, consume the second path token to keep parsing in sync.
        # Prefer the path that exists on disk to avoid passing stale paths to typos.
        if [[ "$status" == *"R"* || "$status" == *"C"* ]]; then
            if IFS= read -r -d '' other_path; then
                if [ ! -e "$filename" ] && [ -e "$other_path" ]; then
                    filename="$other_path"
                fi
            fi
        fi

        # Skip deletions (file may no longer exist).
        if [[ "$status" == *"D"* ]]; then
            continue
        fi

        files+=("$filename")
    done < <(git status --porcelain -z --ignored=no)
    if [ "${#files[@]}" -gt 0 ]; then
        # Exclude typos.toml itself: it intentionally contains allowlisted fragments.
        printf '%s\0' "${files[@]}" | xargs -0 -n100 typos --force-exclude --exclude typos.toml --
    else
        echo "No modified files to spell-check."
    fi

# Testing: fast tests (lib + doc)
test: test-lib test-doc

# Testing: CI suite (lib + integration + examples)
test-ci: test-lib test-integration test-examples

# Testing: comprehensive suite (CI + doc + Python)
test-all: test-ci test-doc test-python
    @echo "✅ All tests passed!"

test-lib:
    cargo test --lib --verbose

test-doc:
    cargo test --doc --verbose

test-integration:
    cargo test --tests --verbose

test-examples:
    cargo test --examples --verbose

test-cli:
    cargo test --test cli --verbose

test-python: _ensure-uv
    uv run pytest

test-release:
    cargo test --release

# File validation
validate-json: _ensure-jq
    #!/usr/bin/env bash
    set -euo pipefail
    files=()
    while IFS= read -r -d '' file; do
        files+=("$file")
    done < <(git ls-files -z '*.json')
    if [ "${#files[@]}" -gt 0 ]; then
        printf '%s\0' "${files[@]}" | xargs -0 -n1 jq empty
    else
        echo "No JSON files found to validate."
    fi

toml-fmt: _ensure-taplo
    #!/usr/bin/env bash
    set -euo pipefail
    files=()
    while IFS= read -r -d '' file; do
        files+=("$file")
    done < <(git ls-files -z '*.toml')

    if [ "${#files[@]}" -gt 0 ]; then
        taplo fmt "${files[@]}"
    else
        echo "No TOML files found to format."
    fi

toml-fmt-check: _ensure-taplo
    #!/usr/bin/env bash
    set -euo pipefail
    files=()
    while IFS= read -r -d '' file; do
        files+=("$file")
    done < <(git ls-files -z '*.toml')

    if [ "${#files[@]}" -gt 0 ]; then
        taplo fmt --check "${files[@]}"
    else
        echo "No TOML files found to check."
    fi

toml-lint: _ensure-taplo
    #!/usr/bin/env bash
    set -euo pipefail
    files=()
    while IFS= read -r -d '' file; do
        files+=("$file")
    done < <(git ls-files -z '*.toml')

    if [ "${#files[@]}" -gt 0 ]; then
        taplo lint "${files[@]}"
    else
        echo "No TOML files found to lint."
    fi

yaml-lint: _ensure-yamllint
    #!/usr/bin/env bash
    set -euo pipefail
    files=()
    while IFS= read -r -d '' file; do
        files+=("$file")
    done < <(git ls-files -z '*.yml' '*.yaml')
    if [ "${#files[@]}" -gt 0 ]; then
        echo "🔍 yamllint (${#files[@]} files)"
        yamllint --strict -c .yamllint "${files[@]}"
    else
        echo "No YAML files found to lint."
    fi

