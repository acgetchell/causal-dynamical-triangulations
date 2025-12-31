# shellcheck disable=SC2148
# Justfile for causal-dynamical-triangulations development workflow
# Install just: https://github.com/casey/just
# Usage: just <command> or just --list

# Use bash with strict error handling for all recipes
set shell := ["bash", "-euo", "pipefail", "-c"]

# Internal helper: ensure uv is installed
_ensure-uv:
    command -v uv >/dev/null || { echo "❌ 'uv' not found. See 'just setup' or https://github.com/astral-sh/uv"; exit 1; }

# Internal helpers: ensure external tooling is installed
_ensure-jq:
    command -v jq >/dev/null || { echo "❌ 'jq' not found. See 'just setup' or install: brew install jq"; exit 1; }

_ensure-npx:
    command -v npx >/dev/null || { echo "❌ 'npx' not found. See 'just setup' or install Node.js (for npx tools): https://nodejs.org"; exit 1; }

_ensure-taplo:
    command -v taplo >/dev/null || { echo "❌ 'taplo' not found. See 'just setup' or install: brew install taplo (or: cargo install taplo-cli)"; exit 1; }

_ensure-shellcheck:
    command -v shellcheck >/dev/null || { echo "❌ 'shellcheck' not found. See 'just setup' or https://www.shellcheck.net"; exit 1; }

_ensure-shfmt:
    command -v shfmt >/dev/null || { echo "❌ 'shfmt' not found. See 'just setup' or https://github.com/mvdan/sh"; exit 1; }

_ensure-actionlint:
    command -v actionlint >/dev/null || { echo "❌ 'actionlint' not found. See 'just setup' or https://github.com/rhysd/actionlint"; exit 1; }

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

bench-compile:
    RUSTFLAGS='-D warnings' cargo bench --workspace --no-run

# Build commands
build:
    cargo build

# Changelog management
changelog: _ensure-uv
    uv run changelog-utils generate

changelog-tag version: _ensure-uv
    uv run changelog-utils tag {{version}}

changelog-update: changelog
    @echo "📝 Changelog updated successfully!"
    @echo "To create a git tag with changelog content for a specific version, run:"
    @echo "  just changelog-tag <version>  # e.g., just changelog-tag v0.4.2"

# Fix (mutating): apply formatters/auto-fixes
fix: toml-fmt fmt python-fix shell-fmt markdown-fix
    @echo "✅ Fixes applied!"

# Check (non-mutating): run all linters/validators
check: lint
    @echo "✅ Checks complete!"

# CI simulation: comprehensive checks + tests (matches .github/workflows/ci.yml)
ci: check test test-integration bench-compile
    @echo "🎯 CI checks complete!"

# CI with performance baseline
ci-baseline tag="ci":
    just ci
    just perf-baseline {{tag}}

# Clean build artifacts
clean:
    cargo clean
    rm -rf target/tarpaulin
    rm -rf coverage_report

# Code quality and formatting
clippy:
    cargo clippy --workspace --all-targets --all-features -- -D warnings -W clippy::pedantic -W clippy::nursery -W clippy::cargo

# Pre-commit workflow: comprehensive validation (checks + tests + release + benches + kani)
commit-check: check test-all test-release bench-compile kani-fast
    @echo "🚀 Ready to commit! All checks passed."

# Coverage analysis
coverage:
    cargo tarpaulin --exclude-files 'benches/**' --exclude-files 'examples/**' --exclude-files 'tests/**' --out Html --output-dir target/tarpaulin
    @echo "📊 Coverage report generated: target/tarpaulin/tarpaulin-report.html"

coverage-report *args: _ensure-uv
    uv run coverage_report {{args}}

# Default recipe shows available commands
default:
    @just --list

# Development workflow: quick format + core Rust checks + tests
dev: fix clippy test
    @echo "⚡ Quick development check complete!"

doc-check:
    RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps --document-private-items

fmt:
    cargo fmt --all

fmt-check:
    cargo fmt --all -- --check

help-workflows:
    @echo "Common Just workflows:"
    @echo "  just fix           # Apply formatters/auto-fixes (mutating)"
    @echo "  just check         # Run linters/validators (non-mutating)"
    @echo "  just dev           # Quick development cycle (fix + clippy + tests)"
    @echo "  just ci            # CI parity (mirrors .github/workflows/ci.yml)"
    @echo "  just commit-check  # Comprehensive pre-commit validation (recommended before pushing)"
    @echo "  just ci-baseline   # CI + save performance baseline"
    @echo ""
    @echo "Testing:"
    @echo "  just test              # Fast tests (lib + doc)"
    @echo "  just test-ci           # CI tests (lib + integration + examples)"
    @echo "  just test-integration  # Integration tests (tests/)"
    @echo "  just test-doc          # Doc tests"
    @echo "  just test-cli          # CLI integration tests only"
    @echo "  just test-python       # Python tests (pytest)"
    @echo "  just test-all          # All tests (CI + doc + Python)"
    @echo "  just test-release      # All tests in release mode"
    @echo "  just coverage          # Generate coverage report"
    @echo ""
    @echo "Quality Check Groups:"
    @echo "  just lint          # All linting (non-mutating checks: code + docs + config)"
    @echo "  just lint-code     # Code checks (Rust, Python, Shell)"
    @echo "  just lint-docs     # Documentation checks (Markdown, Spelling)"
    @echo "  just lint-config   # Configuration checks (JSON, TOML, Actions)"
    @echo ""
    @echo "Formal Verification:"
    @echo "  just kani          # Run all Kani formal verification proofs"
    @echo "  just kani-fast     # Run fast Kani verification (ActionConfig only)"
    @echo ""
    @echo "Performance Analysis:"
    @echo "  just perf-help     # Show performance analysis commands"
    @echo "  just perf-check    # Check for performance regressions"
    @echo "  just perf-baseline # Save current performance as baseline"
    @echo ""
    @echo "Running:"
    @echo "  just run -- <args>  # Run with custom arguments"
    @echo "  just run-example    # Run with example arguments"
    @echo "  just run-simulation # Run basic_simulation.sh example script"
    @echo ""
    @echo "Note: 'just commit-check' includes Kani verification. Run 'just setup' for full environment."
    @echo "Note: Some recipes require external tools. See 'just setup' output."

# Kani formal verification
kani:
    cargo kani

kani-fast:
    cargo kani --harness verify_action_config

# Code linting: Rust (fmt-check, clippy, docs) + Python + Shell scripts
lint-code: fmt-check clippy doc-check python-check shell-check

# Documentation linting: Markdown + spell checking
lint-docs: markdown-check spell-check

# Configuration validation: JSON, TOML, GitHub Actions workflows
lint-config: validate-json toml-lint toml-fmt-check action-lint

# All linting: code + documentation + configuration
lint: lint-code lint-docs lint-config

# Markdown: apply auto-fixes (mutating)
markdown-fix: _ensure-npx
    #!/usr/bin/env bash
    set -euo pipefail
    files=()
    while IFS= read -r -d '' file; do
        files+=("$file")
    done < <(git ls-files -z '*.md')
    if [ "${#files[@]}" -gt 0 ]; then
        printf '%s\0' "${files[@]}" | xargs -0 -n100 npx markdownlint --config .markdownlint.json --fix
    else
        echo "No markdown files found to format."
    fi

# Markdown: lint/check (non-mutating)
markdown-check: _ensure-npx
    #!/usr/bin/env bash
    set -euo pipefail
    files=()
    while IFS= read -r -d '' file; do
        files+=("$file")
    done < <(git ls-files -z '*.md')
    if [ "${#files[@]}" -gt 0 ]; then
        printf '%s\0' "${files[@]}" | xargs -0 -n100 npx markdownlint --config .markdownlint.json
    else
        echo "No markdown files found to check."
    fi

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

# Python code quality
python-fix: _ensure-uv
    uv run ruff check scripts/ --fix
    uv run ruff format scripts/

python-typecheck:
    @echo "🔍 ty (primary)"
    uv run ty check scripts/
    @echo "🔍 mypy (compatibility)"
    uv run mypy scripts/*.py

python-check: _ensure-uv
    uv run ruff format --check scripts/
    uv run ruff check scripts/
    just python-typecheck

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
    for tool in uv actionlint shfmt shellcheck jq node npx taplo cargo-tarpaulin; do
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
                actionlint)
                    echo "    Install: https://github.com/rhysd/actionlint"
                    ;;
                shfmt|shellcheck|jq|taplo)
                    echo "    macOS: brew install $tool"
                    if [ "$tool" = "taplo" ]; then
                        echo "    Or: cargo install taplo-cli"
                    fi
                    ;;
                node|npx)
                    echo "    Install Node.js (for npx/cspell): https://nodejs.org"
                    ;;
                cargo-tarpaulin)
                    echo "    Install: cargo install cargo-tarpaulin"
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
    installed_kani_version="$(cargo kani --version 2>/dev/null || true)"
    if [[ "$installed_kani_version" == "cargo-kani ${kani_version}"* ]]; then
        echo "  ✓ $installed_kani_version"
    else
        cargo install --locked --force --version "${kani_version}" kani-verifier
        cargo kani --version
    fi
    echo ""
    echo "Building project..."
    cargo build
    echo "✅ Setup complete! Run 'just help-workflows' to see available commands."

# Shell scripts: format (mutating)
shell-fmt: _ensure-shfmt
    #!/usr/bin/env bash
    set -euo pipefail
    files=()
    while IFS= read -r -d '' file; do
        files+=("$file")
    done < <(git ls-files -z '*.sh')
    if [ "${#files[@]}" -gt 0 ]; then
        printf '%s\0' "${files[@]}" | xargs -0 -n1 shfmt -w
    else
        echo "No shell files found to format."
    fi

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

# Spell checking with robust bash implementation
spell-check: _ensure-npx
    #!/usr/bin/env bash
    set -euo pipefail
    files=()
    # Use -z for NUL-delimited output to handle filenames with spaces
    while IFS= read -r -d '' status_line; do
        # Extract filename from git status --porcelain -z format
        # Format: XY filename or XY oldname -> newname (for renames)
        if [[ "$status_line" =~ ^..[[:space:]](.*)$ ]]; then
            filename="${BASH_REMATCH[1]}"
            # For renames (format: "old -> new"), take the new filename
            if [[ "$filename" == *" -> "* ]]; then
                filename="${filename#* -> }"
            fi
            files+=("$filename")
        fi
    done < <(git status --porcelain -z --ignored=no)
    if [ "${#files[@]}" -gt 0 ]; then
        printf '%s\0' "${files[@]}" | xargs -0 npx cspell lint --config cspell.json --no-progress --gitignore --cache --exclude cspell.json
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

