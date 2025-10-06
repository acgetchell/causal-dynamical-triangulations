# causal-dynamical-triangulations

[![CI](https://github.com/acgetchell/causal-dynamical-triangulations/actions/workflows/ci.yml/badge.svg)](https://github.com/acgetchell/causal-dynamical-triangulations/actions/workflows/ci.yml)
[![rust-clippy analyze](https://github.com/acgetchell/causal-dynamical-triangulations/actions/workflows/rust-clippy.yml/badge.svg)](https://github.com/acgetchell/causal-dynamical-triangulations/actions/workflows/rust-clippy.yml)
[![Codecov](https://codecov.io/gh/acgetchell/causal-dynamical-triangulations/graph/badge.svg?token=CsbOJBypGC)](https://codecov.io/gh/acgetchell/causal-dynamical-triangulations)
[![Kani CI](https://github.com/acgetchell/causal-dynamical-triangulations/actions/workflows/kani.yml/badge.svg)](https://github.com/acgetchell/causal-dynamical-triangulations/actions/workflows/kani.yml)
[![Audit dependencies](https://github.com/acgetchell/causal-dynamical-triangulations/actions/workflows/audit.yml/badge.svg)](https://github.com/acgetchell/causal-dynamical-triangulations/actions/workflows/audit.yml)

Causal Dynamical Triangulations using Constrained Delaunay Triangulations in Rust

## Introduction

For an introduction to Causal Dynamical Triangulations, see [this paper](https://arxiv.org/abs/hep-th/0105267).

> Causal Dynamical Triangulations (CDT) for quantum gravity in Rust, built on top of fast Delaunay/Voronoi primitives.

<!-- Badges (uncomment/update once workflows are configured for this repo)
[![CI](https://github.com/acgetchell/causal-dynamical-triangulations/actions/workflows/ci.yml/badge.svg)](https://github.com/acgetchell/causal-dynamical-triangulations/actions/workflows/ci.yml)
[![Clippy](https://github.com/acgetchell/causal-dynamical-triangulations/actions/workflows/rust-clippy.yml/badge.svg)](https://github.com/acgetchell/causal-dynamical-triangulations/actions/workflows/rust-clippy.yml)
[![Kani](https://github.com/acgetchell/causal-dynamical-triangulations/actions/workflows/kani.yml/badge.svg)](https://github.com/acgetchell/causal-dynamical-triangulations/actions/workflows/kani.yml)
[![Audit](https://github.com/acgetchell/causal-dynamical-triangulations/actions/workflows/audit.yml/badge.svg)](https://github.com/acgetchell/causal-dynamical-triangulations/actions/workflows/audit.yml)
[![codecov](https://codecov.io/gh/acgetchell/causal-dynamical-triangulations/graph/badge.svg)](https://codecov.io/gh/acgetchell/causal-dynamical-triangulations)
-->

## Overview

This project implements **Causal Dynamical Triangulations (CDT)** in Rust. The goal is to provide a reproducible, well‑tested toolkit for building and evolving
triangulated Lorentzian spacetimes in 2D → 4D, leveraging an existing Delaunay backend for fast geometry queries.

- **Language**: Rust (primary)
- **Geometry backend**: integrates with an external Delaunay crate (e.g., [`delaunay`](https://crates.io/crates/delaunay) / your own `d-delaunay`)
- **Scope**: simulation, ergodic moves, and MCMC over foliated triangulations with causal structure

> **Status**: Early development. API and data structures may change.

## Why Rust for CDT?

- **Safety + performance** for large meshes
- **Property-based testing & model checking** (Kani) for core invariants
- **Ecosystem** support (crates for numerics, rand, rayon, etc.)

## Getting started

Until a crate is published on crates.io, consume from git:

```toml
# Cargo.toml (of your application)
[dependencies]
causal-dynamical-triangulations = { git = "https://github.com/acgetchell/causal-dynamical-triangulations" }
```

> Once published, this will become:
>
> ```toml
> [dependencies]
> causal-dynamical-triangulations = "0.x"
> ```

### Library Usage Example

See [`examples/basic_cdt.rs`](examples/basic_cdt.rs) for a complete working example:

```rust
use causal_dynamical_triangulations::{
    CdtConfig, MetropolisConfig, ActionConfig, MetropolisAlgorithm,
    geometry::CdtTriangulation2D,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create triangulation
    let triangulation = CdtTriangulation2D::new_with_delaunay(10, 1, 2)?;
    
    // Configure simulation
    let metropolis_config = MetropolisConfig::new(1.0, 1000, 100, 10);
    let action_config = ActionConfig::default();
    let mut algorithm = MetropolisAlgorithm::new(metropolis_config, action_config);
    
    // Run simulation
    let results = algorithm.run_simulation_with_backend(triangulation);
    
    println!("Acceptance rate: {:.3}", results.acceptance_rate());
    println!("Average action: {:.3}", results.average_action());
    Ok(())
}
```

### Command Line Interface

The project includes a `cdt-rs` binary for running simulations from the command line:

```bash
# Build the binary
cargo build --release

# Run a basic simulation
./target/release/cdt-rs --vertices 20 --timeslices 10 --steps 2000 --simulate

# With custom physics parameters
./target/release/cdt-rs \
  --vertices 50 --timeslices 12 \
  --temperature 1.5 --coupling-0 0.8 \
  --steps 5000 --simulate
```

**Ready-to-use scripts:**

- [`examples/scripts/basic_simulation.sh`](examples/scripts/basic_simulation.sh) - Simple test run
- [`examples/scripts/parameter_sweep.sh`](examples/scripts/parameter_sweep.sh) - Temperature sweep study
- [`examples/scripts/performance_test.sh`](examples/scripts/performance_test.sh) - Performance benchmarking

See [`docs/CLI_EXAMPLES.md`](docs/CLI_EXAMPLES.md) for comprehensive CLI documentation.

## Roadmap

- [x] Integrate an existing Rust **Delaunay** triangulation library (e.g., [`delaunay`](https://crates.io/crates/delaunay))
- [x] 2D Delaunay triangulation scaffold
- [x] Model‑checking with **[Kani](https://model-checking.github.io/kani/install-guide.html)** for core invariants
- [ ] 1+1 foliation (causal time‑slicing)
- [ ] 2D ergodic moves (Alexander/Pachner moves with causal constraints)
- [ ] 2D Metropolis–Hastings
- [ ] Diffusion‑accelerated MCMC (exploration)
- [ ] Basic visualization hooks (export to common mesh formats)
- [ ] 3D Delaunay + 2+1 foliation + moves + M–H
- [ ] 4D Delaunay + 3+1 foliation + moves + M–H
- [ ] Mass initialization via **Constrained Delaunay** in 3D/4D
- [ ] Shortest paths & geodesic distance
- [ ] Curvature estimates / Einstein tensor (discrete Regge‑like observables)

## Design notes

- **Separation of concerns**: geometry primitives (Delaunay/Voronoi) are decoupled from CDT dynamics.
- **Foliation‑aware data model**: explicit time labels; space‑like vs time‑like edges encoded in types.
- **Testing**: unit + property tests; Kani proofs for invariants (e.g., move reversibility, manifoldness).

## Development

### Prerequisites

- Rust stable (latest)
- Justfile (optional, for task automation)
- (Optional) Kani for model checking

### Common tasks

```bash
# Lint & format
cargo fmt --all
cargo clippy --all-targets -- -D warnings

# Test
cargo test --all

# Run benchmarks
cargo bench

# Kani proofs (if configured)
cargo kani --all
```

### Benchmarking

Comprehensive performance benchmarks are available using criterion:

```bash
# Run all benchmarks
cargo bench

# Specific benchmark categories
cargo bench triangulation_creation
cargo bench edge_counting
cargo bench metropolis_simulation

# Generate HTML reports
cargo bench -- --output-format html
```

See [`benches/README.md`](benches/README.md) for detailed benchmarking documentation.

## References

- J. Ambjørn, J. Jurkiewicz, R. Loll, *Dynamically Triangulating Lorentzian Quantum Gravity*, **Nucl. Phys. B 610** (2001) 347–382. <https://arxiv.org/abs/hep-th/0105267>
- R. Loll, *Quantum Gravity from Causal Dynamical Triangulations: A Review*, **Class. Quantum Grav. 37** (2020) 013002. <https://arxiv.org/abs/1905.08669>
- Ambjørn, Görlich, Jurkiewicz, Loll, *Nonperturbative Quantum Gravity*, **Phys. Rept. 519** (2012) 127–210. <https://arxiv.org/abs/1203.3591>

## Contributing

Contributions are welcome! Please open an issue to discuss proposed features or design changes.

## License

This project’s license is specified in `LICENSE`

---

### Project history / notes

- Originally experimented under names like `cdt-rs` and `cdt`; this repository consolidates CDT work with a consistent name and roadmap.
