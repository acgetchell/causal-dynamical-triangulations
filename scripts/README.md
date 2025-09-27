# Scripts Directory

This directory contains utility scripts for building, testing, and benchmarking
the causal-dynamical-triangulations library.

**Note**: Tests for the Python utilities are located in `scripts/tests/` and
can be run with `uv run pytest`.

## Prerequisites

Before running these scripts, ensure you have the following dependencies installed:

### Python 3.11+ (Required)

```bash
# Install Python 3.11+ and uv package manager
brew install python@3.11 uv  # macOS with Homebrew
# or follow installation instructions for your platform
```

### Additional Dependencies

```bash
# macOS (using Homebrew)
brew install jq findutils coreutils

# Ubuntu/Debian 
sudo apt-get install -y jq
```

## Scripts Overview

### Python Utilities (Primary)

All Python utilities require Python 3.11+ and support `--help` for detailed
usage. The project uses modern Python with comprehensive utilities for
benchmarking, analysis, and project management focused on causal dynamical
triangulation research.

**Project Focus**: This scripts directory is configured for utilities specific
to causal dynamical triangulations research, including:

- Performance analysis of triangulation algorithms
- Data processing for causal structure analysis  
- Research workflow automation
- Visualization utilities for spacetime triangulations

### Module Organization

- **Shared utilities**: Security-hardened subprocess and common functionality
- **Benchmarking tools**: Performance analysis for CDT algorithms
- **Analysis utilities**: Data processing for causal structure research
- **Visualization tools**: Plotting and visualization for research results

This modular design ensures code reuse and maintainability across all utilities.

## Development Integration

### CI/CD Integration

The scripts are integrated with GitHub Actions workflows for:

- Automated testing of Python utilities
- Performance regression detection
- Code quality assurance

### Code Quality

All Python scripts are automatically checked in CI:

```bash
# Format Python code
uvx ruff format scripts/

# Lint and auto-fix Python code
uvx ruff check --fix scripts/

# Run tests
uv run pytest
```

## Error Handling and Troubleshooting

### Common Issues

1. **Missing Dependencies**: Install required packages using your system's
   package manager
2. **Permission Errors**: Ensure scripts are executable where needed
3. **Path Issues**: Run scripts from the project root directory or use
   appropriate relative paths
4. **Python Version**: Ensure Python 3.11+ is installed and available

### Exit Codes

- `0` - Success
- `1` - General error
- `2` - Missing dependency
- `3` - File/directory not found

### Debug Mode

```bash
# For Python scripts, use built-in help and verbose options
uv run --help
```

## Script Maintenance

All scripts follow consistent patterns:

### Python Scripts

- **Modern Python**: Python 3.11+ with type hints
- **Security**: Secure subprocess execution patterns
- **Error Handling**: Custom exception classes with clear error messages
- **Configuration**: Uses `pyproject.toml` for dependencies and tool configuration
- **Code Quality**: Comprehensive linting and formatting standards

When modifying scripts, maintain these patterns for consistency and reliability.

## Future Extensions

This scripts directory is designed to grow with the project's needs. Potential
future additions include:

- Monte Carlo simulation utilities
- Spacetime geometry analysis tools
- Quantum gravity computation helpers
- Research data management utilities
- Publication and plotting automation

The modular structure supports easy addition of new utilities while maintaining
code quality and consistency.
