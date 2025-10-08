# CDT-RS Command Line Interface Examples

This document provides examples for using the `cdt-rs` binary, the command-line interface for Causal Dynamical Triangulations simulations.

## Basic Usage

The `cdt-rs` binary accepts various command-line arguments to configure and run CDT simulations.

### Quick Start

```bash
# Basic 2D CDT simulation with default parameters
./target/release/cdt-rs --vertices 10 --timeslices 5

# Run with custom temperature and steps
./target/release/cdt-rs --vertices 20 --timeslices 10 --temperature 1.5 --steps 2000
```

## Command Line Arguments

### Required Arguments

- `--vertices <N>`: Number of vertices in the triangulation (minimum 3)
- `--timeslices <N>`: Number of time slices in the CDT foliation (minimum 1)

### Optional Simulation Parameters

- `--dimension <D>`: Dimensionality (2-3, default: 2)
- `--temperature <T>`: Temperature for Metropolis algorithm (default: 1.0)
- `--steps <N>`: Number of Monte Carlo steps (default: 1000)
- `--thermalization-steps <N>`: Thermalization steps before measurements (default: 100)
- `--measurement-frequency <N>`: Take measurement every N steps (default: 10)

### Physics Parameters

- `--coupling-0 <κ₀>`: Coupling constant for vertices (default: 1.0)
- `--coupling-2 <κ₂>`: Coupling constant for triangles (default: 1.0)
- `--cosmological-constant <λ>`: Cosmological constant (default: 0.1)

### Additional Options

- `--simulate`: Run full Monte Carlo simulation (default: false, just creates triangulation)

## Example Usage Scenarios

### 1. Small Test Simulation

```bash
# Quick test with minimal parameters
./target/release/cdt-rs --vertices 5 --timeslices 2 --simulate
```

**Expected Output:**

- Creates a 5-vertex, 2-timeslice triangulation
- Runs default 1000 Monte Carlo steps
- Reports simulation statistics

### 2. Medium-Scale Physics Study

```bash
# Medium triangulation for physics exploration  
./target/release/cdt-rs \
  --vertices 50 \
  --timeslices 10 \
  --temperature 1.2 \
  --steps 5000 \
  --thermalization-steps 500 \
  --measurement-frequency 25 \
  --simulate
```

**Use Case:** Study phase transitions or scaling behavior

### 3. High-Temperature Simulation

```bash
# High temperature (classical limit)
./target/release/cdt-rs \
  --vertices 30 \
  --timeslices 8 \
  --temperature 10.0 \
  --steps 3000 \
  --simulate
```

**Use Case:** Explore classical geometry limit

### 4. Low-Temperature Simulation

```bash
# Low temperature (quantum regime)
./target/release/cdt-rs \
  --vertices 25 \
  --timeslices 12 \
  --temperature 0.5 \
  --steps 8000 \
  --thermalization-steps 1000 \
  --simulate
```

**Use Case:** Study quantum fluctuations and crumpled phase

### 5. Custom Physics Parameters

```bash
# Modified coupling constants
./target/release/cdt-rs \
  --vertices 40 \
  --timeslices 8 \
  --coupling-0 0.8 \
  --coupling-2 1.2 \
  --cosmological-constant 0.05 \
  --simulate
```

**Use Case:** Explore modified gravity or different action formulations

### 6. Triangulation-Only Mode

```bash
# Generate triangulation without simulation
./target/release/cdt-rs --vertices 100 --timeslices 20
```

**Use Case:** Generate initial configurations for other analysis tools

## Advanced Usage Patterns

### Batch Processing with Shell Scripts

Create a script to run parameter sweeps:

```bash
#!/bin/bash
# parameter_sweep.sh

for temp in 0.5 1.0 1.5 2.0 2.5; do
    echo "Running simulation at temperature $temp"
    ./target/release/cdt-rs \
        --vertices 30 \
        --timeslices 10 \
        --temperature $temp \
        --steps 2000 \
        --simulate \
        > "results_T${temp}.log" 2>&1
done
```

### Performance Testing

```bash
# Large simulation for performance testing
./target/release/cdt-rs \
  --vertices 200 \
  --timeslices 25 \
  --steps 10000 \
  --measurement-frequency 100 \
  --simulate
```

### Logging and Output

Enable detailed logging:

```bash
# Set log level for detailed output
RUST_LOG=debug ./target/release/cdt-rs --vertices 10 --timeslices 5 --simulate

# Log only errors and warnings
RUST_LOG=warn ./target/release/cdt-rs --vertices 50 --timeslices 10 --simulate

# Save output to file
./target/release/cdt-rs --vertices 25 --timeslices 8 --simulate > simulation.log 2>&1
```

## Expected Output Format

### Successful Run

```text
[INFO] Dimensionality: 2
[INFO] Number of vertices: 10
[INFO] Time slices: 5
[INFO] Starting CDT simulation with backend...
[INFO] Temperature: 1.0
[INFO] Total steps: 1000
[INFO] Thermalization steps: 100
[INFO] Simulation completed in 45.23ms
[INFO] CDT simulation completed successfully
```

### Error Cases

```bash
# Invalid parameters
./target/release/cdt-rs --vertices 2 --timeslices 1
# Error: vertices must be >= 3

# Unsupported dimension
./target/release/cdt-rs --vertices 10 --timeslices 5 --dimension 4
# Error: unsupported dimension
```

## Performance Considerations

### Memory Usage

- Small (≤20 vertices): <10 MB
- Medium (20-100 vertices): 10-100 MB  
- Large (100+ vertices): 100+ MB

### Runtime

- Test simulations (1000 steps): seconds
- Physics studies (10000+ steps): minutes
- Large systems: hours

### Optimization Tips

1. **Use release builds** for performance: `cargo build --release`
2. **Adjust measurement frequency** for long runs
3. **Monitor memory** for large vertex counts
4. **Use appropriate thermalization** (typically 10-20% of total steps)

## Troubleshooting

### Common Issues

1. **Binary not found**

   ```bash
   cargo build --release
   ./target/release/cdt-rs --help
   ```

2. **Insufficient memory**
   - Reduce vertex count or steps
   - Monitor system resources

3. **Slow performance**
   - Ensure using release build
   - Check system load
   - Consider reducing measurement frequency

4. **Parameter validation errors**
   - Check minimum values (vertices ≥ 3, timeslices ≥ 1)
   - Verify dimension is 2 or 3

## Integration with Other Tools

### Data Analysis

```bash
# Pipe output to analysis tools
./target/release/cdt-rs --vertices 50 --timeslices 10 --simulate | \
  python analysis_script.py
```

### Automation

```bash
# Use in makefiles or CI/CD
make run-simulation: 
 ./target/release/cdt-rs --vertices $(VERTICES) --timeslices $(SLICES) --simulate
```

## Help and Documentation

```bash
# Display all available options
./target/release/cdt-rs --help

# Version information  
./target/release/cdt-rs --version
```

This CLI interface provides a powerful way to explore CDT physics and test different simulation parameters efficiently from the command line.
