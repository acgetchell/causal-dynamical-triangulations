# shellcheck disable=SC2148
# Justfile for causal-dynamical-triangulations development workflow
# Install just: https://github.com/casey/just
# Usage: just <command> or just --list

# Default recipe shows available commands
default:
    @just --list

# Development setup
setup:
    @echo "Setting up development environment..."
    @echo "Note: Rust toolchain and components are managed by rust-toolchain.toml"
    @echo "Installing Kani verifier for formal verification (required for CI simulation)..."
    cargo install --locked kani-verifier
    cargo kani setup
    @echo "Building project..."
    cargo build

# Code quality and formatting
fmt:
    cargo fmt --all

clippy:
    cargo clippy --workspace --all-targets --all-features -- -D warnings -W clippy::pedantic -W clippy::nursery -W clippy::cargo

doc-check:
    RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --document-private-items

lint: fmt clippy doc-check

# Shell and markdown quality
shell-lint:
    git ls-files -z '*.sh' | xargs -0 -r -n1 shfmt -w
    git ls-files -z '*.sh' | xargs -0 -r -n4 shellcheck -x
    @# Note: justfiles are not shell scripts and are excluded from shellcheck

markdown-lint:
    git ls-files -z '*.md' | xargs -0 -r -n100 npx markdownlint --config .markdownlint.json --fix

# Spell checking
spell-check:
    files="$(git status --porcelain | awk '{print $2}')"; \
    if [ -n "$files" ]; then \
        npx cspell lint --config cspell.json --no-progress --gitignore --cache --exclude cspell.json $files; \
    else \
        echo "No modified files to spell-check."; \
    fi

# File validation
validate-json:
    git ls-files -z '*.json' | xargs -0 -r -n1 jq empty

validate-toml:
    git ls-files -z '*.toml' | xargs -0 -r -I {} sh -c 'cd scripts && uv run python -c "import tomllib; tomllib.load(open(\"../{}\", \"rb\")); print(\"{} is valid TOML\")"'

# Comprehensive quality check
quality: fmt clippy doc-check shell-lint markdown-lint spell-check validate-json validate-toml
    @echo "✅ All quality checks passed!"

# Testing
test:
    cargo test --verbose

test-release:
    cargo test --release

test-cli:
    cargo test --test cli --verbose

kani:
    cargo kani

kani-fast:
    cargo kani --harness verify_action_config

test-all: test test-cli
    @echo "✅ All tests passed!"

# Binary execution
run *args:
    cargo run --bin cdt-rs {{args}}

run-release *args:
    cargo run --release --bin cdt-rs {{args}}

# Example runs
run-example:
    cargo run --bin cdt-rs -- -v 32 -t 3

# Build
build:
    cargo build

build-release:
    cargo build --release

# Benchmarks
bench-compile:
    cargo bench --workspace --no-run

bench:
    cargo bench --workspace

# Coverage analysis
coverage:
    cargo tarpaulin --exclude-files 'benches/**' --exclude-files 'examples/**' --exclude-files 'tests/**' --out Html --output-dir target/tarpaulin
    @echo "📊 Coverage report generated: target/tarpaulin/tarpaulin-report.html"

# Performance analysis
perf-baseline tag="":
    #!/usr/bin/env bash
    if [ -n "{{tag}}" ]; then
        uv run performance-analysis --save-baseline --tag "{{tag}}"
    else
        uv run performance-analysis --save-baseline
    fi

perf-check threshold="10.0":
    uv run performance-analysis --threshold {{threshold}}

perf-report file="":
    #!/usr/bin/env bash
    if [ -n "{{file}}" ]; then
        uv run performance-analysis --report "{{file}}"
    else
        timestamp=$(date +"%Y%m%d_%H%M%S")
        uv run performance-analysis --report "performance_report_${timestamp}.md"
        echo "📄 Report saved to: performance_report_${timestamp}.md"
    fi

perf-trends days="7":
    uv run performance-analysis --trends {{days}}

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

# Pre-commit workflow (recommended before pushing)
commit-check: quality test-all
    @echo "🚀 Ready to commit! All checks passed."

# CI simulation (run what CI runs)
ci: quality test-release bench-compile kani-fast
    @echo "🎯 CI simulation complete!"

# CI with performance baseline (for main branch)
ci-baseline tag="ci":
    just ci
    just perf-baseline {{tag}}

# Development workflow
dev: fmt clippy test
    @echo "⚡ Quick development check complete!"

# Clean build artifacts
clean:
    cargo clean
    rm -rf target/tarpaulin
    rm -rf coverage_report

# Help with common workflows
help-workflows:
    @echo "Common Just workflows:"
    @echo "  just setup         # Set up development environment (includes Kani)"
    @echo "  just dev           # Quick development cycle (format, lint, test)"
    @echo "  just commit-check  # Full pre-commit checks"
    @echo "  just ci            # Simulate CI pipeline (requires Kani)"
    @echo "  just ci-baseline   # CI + save performance baseline"
    @echo "  just quality       # All quality checks"
    @echo "  just test-all      # All tests"
    @echo "  just kani          # Run all Kani formal verification proofs"
    @echo "  just kani-fast     # Run fast Kani verification (ActionConfig only)"
    @echo "  just coverage      # Generate coverage report"
    @echo "  just run-example   # Run with example arguments"
    @echo "  just run -- <args> # Run with custom arguments"
    @echo ""
    @echo "Performance Analysis:"
    @echo "  just perf-help     # Show performance analysis commands"
    @echo "  just perf-check    # Check for performance regressions"
    @echo "  just perf-baseline # Save current performance as baseline"
    @echo ""
    @echo "Note: 'just ci' requires Kani verifier. Run 'just setup' for full environment."
