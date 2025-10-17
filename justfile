# shellcheck disable=SC2148
# Justfile for causal-dynamical-triangulations development workflow
# Install just: https://github.com/casey/just
# Usage: just <command> or just --list

# GitHub Actions workflow validation
action-lint:
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
    cargo bench --workspace --no-run

# Build commands
build:
    cargo build

build-release:
    cargo build --release

# Changelog management
changelog:
    uv run changelog-utils generate

changelog-tag version:
    uv run changelog-utils tag {{version}}

changelog-update: changelog
    @echo "📝 Changelog updated successfully!"
    @echo "To create a git tag with changelog content for a specific version, run:"
    @echo "  just changelog-tag <version>  # e.g., just changelog-tag v0.4.2"

# CI simulation: quality checks + release tests + benchmark compilation + Kani fast
ci: quality test-release bench-compile kani-fast
    @echo "🎯 CI simulation complete!"

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

# Pre-commit workflow: quality + all tests (most comprehensive validation)
commit-check: quality test-all
    @echo "🚀 Ready to commit! All checks passed."

# Coverage analysis
coverage:
    cargo tarpaulin --exclude-files 'benches/**' --exclude-files 'examples/**' --exclude-files 'tests/**' --out Html --output-dir target/tarpaulin
    @echo "📊 Coverage report generated: target/tarpaulin/tarpaulin-report.html"

coverage-report *args:
    uv run coverage_report {{args}}

# Default recipe shows available commands
default:
    @just --list

# Development workflow: quick format, lint, and test cycle
dev: fmt clippy test
    @echo "⚡ Quick development check complete!"

doc-check:
    RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps --document-private-items

fmt:
    cargo fmt --all

help-workflows:
    @echo "Common Just workflows:"
    @echo "  just dev           # Quick development cycle (format, lint, test)"
    @echo "  just quality       # All quality checks + tests (comprehensive)"
    @echo "  just ci            # CI simulation (quality + release tests + bench compile + kani-fast)"
    @echo "  just commit-check  # Pre-commit validation (quality + all tests) - most thorough"
    @echo "  just ci-baseline   # CI + save performance baseline"
    @echo ""
    @echo "Testing:"
    @echo "  just test          # Rust tests (debug mode)"
    @echo "  just test-all      # All tests (Rust + CLI, debug mode)"
    @echo "  just test-cli      # CLI integration tests"
    @echo "  just test-release  # All tests in release mode"
    @echo "  just coverage      # Generate coverage report"
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
    @echo "Note: 'just ci' requires Kani verifier. Run 'just setup' for full environment."
    @echo "Note: Some recipes require external tools. See 'just setup' output."

# Kani formal verification
kani:
    cargo kani

kani-fast:
    cargo kani --harness verify_action_config

# Code linting: Rust (fmt, clippy, docs) + Python (ruff) + Shell scripts
lint-code: fmt clippy doc-check python-lint shell-lint

# Documentation linting: Markdown + spell checking
lint-docs: markdown-lint spell-check

# Configuration validation: JSON, TOML, GitHub Actions workflows
lint-config: validate-json validate-toml action-lint

# All linting: code + documentation + configuration
lint: lint-code lint-docs lint-config

# Shell and markdown quality
markdown-lint:
    #!/usr/bin/env bash
    set -euo pipefail
    files=()
    while IFS= read -r -d '' file; do
        files+=("$file")
    done < <(git ls-files -z '*.md')
    if [ "${#files[@]}" -gt 0 ]; then
        printf '%s\0' "${files[@]}" | xargs -0 -n100 npx markdownlint --config .markdownlint.json --fix
    else
        echo "No markdown files found to lint."
    fi

perf-baseline tag="":
    #!/usr/bin/env bash
    set -euo pipefail
    tag_value="{{tag}}"
    if [ -n "$tag_value" ]; then
        uv run performance-analysis --save-baseline --tag "$tag_value"
    else
        uv run performance-analysis --save-baseline
    fi

perf-check threshold="10.0":
    uv run performance-analysis --threshold {{threshold}}

perf-compare file:
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

perf-report file="":
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

perf-trends days="7":
    uv run performance-analysis --trends {{days}}

# Python code quality
python-lint:
    uv run ruff check scripts/ --fix
    uv run ruff format scripts/

# Comprehensive quality check: all linting + all tests
quality: lint-code lint-docs lint-config test-all
    @echo "✅ All quality checks and tests passed!"

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

# Development setup
setup:
    @echo "Setting up causal-dynamical-triangulations development environment..."
    @echo "Note: Rust toolchain and components are managed by rust-toolchain.toml"
    @echo ""
    @echo "Installing Rust components..."
    rustup component add clippy rustfmt rust-docs rust-src
    @echo ""
    @echo "Additional tools required (install separately):"
    @echo "  - uv: https://github.com/astral-sh/uv"
    @echo "  - actionlint: https://github.com/rhysd/actionlint"
    @echo "  - shfmt, shellcheck: via package manager (brew install shfmt shellcheck)"
    @echo "  - jq: via package manager (brew install jq)"
    @echo "  - Node.js (for npx/cspell): https://nodejs.org"
    @echo "  - cargo-tarpaulin: cargo install cargo-tarpaulin"
    @echo ""
    @echo "Installing Python tooling (ruff and related dependencies)..."
    uv sync --group dev
    @echo ""
    @echo "Installing Kani verifier for formal verification (required for CI simulation)..."
    cargo install --locked kani-verifier
    cargo kani setup
    @echo ""
    @echo "Building project..."
    cargo build
    @echo "✅ Setup complete! Run 'just help-workflows' to see available commands."

shell-lint:
    #!/usr/bin/env bash
    set -euo pipefail
    files=()
    while IFS= read -r -d '' file; do
        files+=("$file")
    done < <(git ls-files -z '*.sh')
    if [ "${#files[@]}" -gt 0 ]; then
        printf '%s\0' "${files[@]}" | xargs -0 -n1 shfmt -w
        printf '%s\0' "${files[@]}" | xargs -0 -n4 shellcheck -x
    else
        echo "No shell files found to lint."
    fi
    # Note: justfiles are not shell scripts and are excluded from shellcheck

# Spell checking with robust bash implementation
spell-check:
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

# Testing
test:
    cargo test --verbose

test-all: test test-cli
    @echo "✅ All tests passed!"

test-cli:
    cargo test --test cli --verbose

test-release:
    cargo test --release

# File validation
validate-json:
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

validate-toml:
    #!/usr/bin/env bash
    set -euo pipefail
    files=()
    while IFS= read -r -d '' file; do
        files+=("$file")
    done < <(git ls-files -z '*.toml')
    if [ "${#files[@]}" -gt 0 ]; then
        printf '%s\0' "${files[@]}" | xargs -0 -I {} uv run python -c "import tomllib; tomllib.load(open('{}', 'rb')); print('{} is valid TOML')"
    else
        echo "No TOML files found to validate."
    fi
