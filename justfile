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
    rustup component add clippy rustfmt rust-docs rust-src
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
    cargo test --lib --verbose
    cargo test --doc --verbose

test-release:
    cargo test --release

test-cli:
    cargo test --test cli --verbose

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

# Pre-commit workflow (recommended before pushing)
check: quality test-all
    @echo "🚀 Ready to commit! All checks passed."

# CI simulation (run what CI runs)
ci: quality test-release bench-compile
    @echo "🎯 CI simulation complete!"

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
    @echo "  just dev           # Quick development cycle (format, lint, test)"
    @echo "  just check         # Full pre-commit checks"
    @echo "  just ci            # Simulate CI pipeline"
    @echo "  just quality       # All quality checks"
    @echo "  just test-all      # All tests"
    @echo "  just coverage      # Generate coverage report"
    @echo "  just run-example   # Run with example arguments"
    @echo "  just run -- <args> # Run with custom arguments"