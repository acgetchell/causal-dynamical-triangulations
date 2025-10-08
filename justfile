# shellcheck disable=SC2148
# Justfile for causal-dynamical-triangulations development workflow
# Install just: https://github.com/casey/just
# Usage: just <command> or just --list

action-lint:
    git ls-files -z '.github/workflows/*.yml' '.github/workflows/*.yaml' | xargs -0 -r actionlint

bench:
    cargo bench --workspace

bench-compile:
    cargo bench --workspace --no-run

build:
    cargo build

build-release:
    cargo build --release

changelog-update:
    uv run changelog-utils

ci: quality test-release bench-compile kani-fast
    @echo "🎯 CI simulation complete!"

ci-baseline tag="ci":
    just ci
    just perf-baseline {{tag}}

clean:
    cargo clean
    rm -rf target/tarpaulin
    rm -rf coverage_report

clippy:
    cargo clippy --workspace --all-targets --all-features -- -D warnings -W clippy::pedantic -W clippy::nursery -W clippy::cargo

commit-check: quality test-all
    @echo "🚀 Ready to commit! All checks passed."

coverage:
    cargo tarpaulin --exclude-files 'benches/**' --exclude-files 'examples/**' --exclude-files 'tests/**' --out Html --output-dir target/tarpaulin
    @echo "📊 Coverage report generated: target/tarpaulin/tarpaulin-report.html"

coverage-report *args:
    uv run coverage_report {{args}}

default:
    @just --list

dev: fmt clippy test
    @echo "⚡ Quick development check complete!"

doc-check:
    RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --document-private-items

fmt:
    cargo fmt --all

help-workflows:
    @echo "Common Just workflows:"
    @echo "  just action-lint   # Lint all GitHub workflows with actionlint"
    @echo "  just ci            # Simulate CI pipeline (requires Kani)"
    @echo "  just ci-baseline   # CI + save performance baseline"
    @echo "  just commit-check  # Full pre-commit checks"
    @echo "  just coverage      # Generate coverage report"
    @echo "  just dev           # Quick development cycle (format, lint, test)"
    @echo "  just kani          # Run all Kani formal verification proofs"
    @echo "  just kani-fast     # Run fast Kani verification (ActionConfig only)"
    @echo "  just python-lint   # Lint and format Python scripts with Ruff"
    @echo "  just quality       # All quality checks"
    @echo "  just run -- <args> # Run with custom arguments"
    @echo "  just run-example   # Run with example arguments"
    @echo "  just setup         # Set up development environment (includes Kani)"
    @echo "  just test-all      # All tests"
    @echo ""
    @echo "Performance Analysis:"
    @echo "  just perf-help     # Show performance analysis commands"
    @echo "  just perf-check    # Check for performance regressions"
    @echo "  just perf-baseline # Save current performance as baseline"
    @echo ""
    @echo "Note: 'just ci' requires Kani verifier. Run 'just setup' for full environment."
    @echo "Note: Some recipes require external tools (uv, actionlint, jq, etc.). See 'just setup' output."

kani:
    cargo kani

kani-fast:
    cargo kani --harness verify_action_config

lint: fmt clippy doc-check

markdown-lint:
    git ls-files -z '*.md' | xargs -0 -r -n100 npx markdownlint --config .markdownlint.json --fix

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

python-lint:
    uv run ruff check scripts/ --fix
    uv run ruff format scripts/

quality: fmt clippy doc-check python-lint shell-lint markdown-lint spell-check validate-json validate-toml action-lint
    @echo "✅ All quality checks passed!"

run *args:
    cargo run --bin cdt-rs {{args}}

run-example:
    cargo run --bin cdt-rs -- -v 32 -t 3

run-release *args:
    cargo run --release --bin cdt-rs {{args}}

setup:
    @echo "Setting up development environment..."
    @echo "Note: Rust toolchain and components are managed by rust-toolchain.toml"
    @echo ""
    @echo "Additional tools required (install separately):"
    @echo "  - uv: https://github.com/astral-sh/uv"
    @echo "  - actionlint: https://github.com/rhysd/actionlint"
    @echo "  - shfmt, shellcheck: via package manager"
    @echo "  - jq: via package manager"
    @echo "  - Node.js (for npx/cspell): https://nodejs.org"
    @echo "  - cargo-tarpaulin: cargo install cargo-tarpaulin"
    @echo "  - python-lint tooling: installed via 'uv sync --group dev'"
    @echo ""
    @echo "Installing Python tooling (ruff and related dependencies)..."
    uv sync --group dev
    @echo ""
    @echo "Installing Kani verifier for formal verification (required for CI simulation)..."
    cargo install --locked kani-verifier
    cargo kani setup
    @echo "Building project..."
    cargo build

shell-lint:
    git ls-files -z '*.sh' | xargs -0 -r -n1 shfmt -w
    git ls-files -z '*.sh' | xargs -0 -r -n4 shellcheck -x
    @# Note: justfiles are not shell scripts and are excluded from shellcheck

spell-check:
	#!/usr/bin/env bash
	set -euo pipefail
	files=()
	while IFS= read -r -d '' file; do
		files+=("$file")
	done < <(git diff --name-only -z HEAD)
	if [ "${#files[@]}" -gt 0 ]; then
		printf '%s\0' "${files[@]}" | xargs -0 npx cspell lint --config cspell.json --no-progress --gitignore --cache --exclude cspell.json
	else
		echo "No modified files to spell-check."
	fi

test:
    cargo test --verbose

test-all: test test-cli
    @echo "✅ All tests passed!"

test-cli:
    cargo test --test cli --verbose

test-release:
    cargo test --release

validate-json:
    git ls-files -z '*.json' | xargs -0 -r -n1 jq empty

validate-toml:
    git ls-files -z '*.toml' | xargs -0 -r -I {} uv run python -c "import tomllib; tomllib.load(open('{}', 'rb')); print('{} is valid TOML')"
