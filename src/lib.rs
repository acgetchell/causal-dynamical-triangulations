#![allow(clippy::multiple_crate_versions)]
#![warn(missing_docs)]

//! Causal Dynamical Triangulations library for quantum gravity simulations.
//!
//! This library implements Causal Dynamical Triangulations (CDT) in 2D, providing
//! the necessary tools for Monte Carlo simulations of discrete spacetime geometries.
//!
//! # Key Features
//!
//! - Integration with delaunay crate for proper Delaunay triangulations
//! - 2D Regge Action calculation for CDT
//! - Standard ergodic moves: (2,2), (1,3), and edge flips
//! - Metropolis-Hastings algorithm with configurable temperature
//! - Comprehensive statistics and measurement collection
//!
//! # Example
//!
//! ```rust,no_run
//! // Example would require command line arguments, so we skip execution
//! use causal_dynamical_triangulations::{Config, run};
//! // Config::build() requires CLI arguments, so this is marked no_run
//! ```

use clap::Parser;

// Module declarations (avoiding mod.rs files)
/// Error types for the CDT library.
pub mod errors;

/// Utility functions for random number generation and mathematical operations.
pub mod util;

/// Geometry abstraction layer for CDT simulations.
///
/// This module provides trait-based geometry operations that isolate CDT algorithms
/// from specific geometry implementations.
pub mod geometry {
    /// CDT-agnostic mesh data structures.
    pub mod mesh;
    /// High-level triangulation operations.
    pub mod operations;
    /// Core geometry traits for CDT abstraction.
    pub mod traits;

    /// Geometry backend implementations.
    pub mod backends {
        /// Delaunay backend - wraps the delaunay crate.
        pub mod delaunay;

        /// Mock backend for testing.
        pub mod mock;
    }
}

/// Causal Dynamical Triangulations implementation modules.
pub mod cdt {
    /// Action calculation for CDT simulations.
    pub mod action;
    /// Ergodic moves for triangulation modifications.
    pub mod ergodic_moves;
    /// Metropolis-Hastings algorithm implementation.
    pub mod metropolis;
    /// CDT triangulation wrapper.
    pub mod triangulation;
}

/// Triangulation data structures and algorithms (DEPRECATED - use `cdt::triangulation`).
pub mod triangulations {
    /// Unified triangulation module with generic Tds support.
    pub mod triangulation;
}

// Re-exports for convenience
pub use cdt::action::{ActionConfig, calculate_regge_action_2d};
pub use cdt::ergodic_moves::{ErgodicsSystem, MoveResult, MoveType};
pub use cdt::metropolis::{MetropolisAlgorithm, MetropolisConfig, SimulationResults};
pub use errors::{CdtError, CdtResult};

// New trait-based triangulation (RECOMMENDED for new code)
pub use cdt::triangulation::CdtTriangulation;

// Legacy triangulation types (DEPRECATED - migrate to CdtTriangulation)
#[deprecated(
    since = "0.1.0",
    note = "Use CdtTriangulation with trait-based backends instead"
)]
#[allow(deprecated)]
pub use triangulations::triangulation::{CausalTriangulation, CausalTriangulation2D};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
/// Configuration options for the `cdt-rs` crate.
pub struct Config {
    /// Dimensionality of the triangulation
    #[arg(short, long, value_parser = clap::value_parser!(u8).range(2..4))]
    dimension: Option<u8>,

    /// Number of vertices
    #[arg(short, long, value_parser = clap::value_parser!(u32).range(3..))]
    vertices: u32,

    /// Number of timeslices
    #[arg(short, long, value_parser = clap::value_parser!(u32).range(1..))]
    timeslices: u32,

    /// Temperature for Metropolis algorithm
    #[arg(long, default_value = "1.0")]
    temperature: f64,

    /// Number of Monte Carlo steps
    #[arg(long, default_value = "1000")]
    steps: u32,

    /// Number of thermalization steps
    #[arg(long, default_value = "100")]
    thermalization_steps: u32,

    /// Measurement frequency (take measurement every N steps)
    #[arg(long, default_value = "10")]
    measurement_frequency: u32,

    /// Coupling constant κ₀ for vertices
    #[arg(long, default_value = "1.0")]
    coupling_0: f64,

    /// Coupling constant κ₂ for triangles
    #[arg(long, default_value = "1.0")]
    coupling_2: f64,

    /// Cosmological constant λ
    #[arg(long, default_value = "0.1")]
    cosmological_constant: f64,

    /// Run full CDT simulation (default: false, just generate triangulation)
    #[arg(long, default_value = "false")]
    simulate: bool,
}

impl Config {
    /// Builds a new instance of `Config`.
    #[must_use]
    pub fn build() -> Self {
        Self::parse()
    }

    /// Creates a `MetropolisConfig` from this Config.
    #[must_use]
    pub const fn to_metropolis_config(&self) -> MetropolisConfig {
        MetropolisConfig::new(
            self.temperature,
            self.steps,
            self.thermalization_steps,
            self.measurement_frequency,
        )
    }

    /// Creates an `ActionConfig` from this Config.
    #[must_use]
    pub const fn to_action_config(&self) -> ActionConfig {
        ActionConfig::new(self.coupling_0, self.coupling_2, self.cosmological_constant)
    }
}

/// Runs the triangulation with the given configuration (LEGACY - use `run_with_backend` instead).
///
/// **DEPRECATED**: This function uses the old direct Tds approach.
/// New code should use `run_with_backend()` for better abstraction.
///
/// This function can either generate a simple triangulation or run a full CDT simulation
/// depending on the `simulate` flag in the configuration.
#[deprecated(since = "0.1.0", note = "Use run_with_backend() instead")]
///
/// # Arguments
///
/// * `config` - Configuration parameters for the triangulation/simulation
///
/// # Returns
///
/// A `SimulationResults` struct containing the results of the operation,
/// including the final triangulation and any measurements taken.
///
/// # Errors
///
/// Returns [`CdtError::UnsupportedDimension`] if an unsupported dimension (not 2D) is specified.
/// Returns triangulation generation errors from the underlying triangulation creation.
pub fn run(config: &Config) -> CdtResult<SimulationResults<f64, i32, i32, 2>> {
    let vertices = config.vertices;
    let timeslices = config.timeslices;

    if let Some(dim) = config.dimension
        && dim != 2
    {
        return Err(CdtError::UnsupportedDimension(dim.into()));
    }

    log::info!("Dimensionality: {}", config.dimension.unwrap_or(2));
    log::info!("Number of vertices: {vertices}");
    log::info!("Number of timeslices: {timeslices}");

    // Create initial triangulation
    #[allow(deprecated)]
    let triangulation = CausalTriangulation::new(vertices, timeslices, 2)?;
    triangulation.print_summary();

    if config.simulate {
        // Run full CDT simulation
        let metropolis_config = config.to_metropolis_config();
        let action_config = config.to_action_config();

        let mut algorithm = MetropolisAlgorithm::new(metropolis_config, action_config);
        let results = algorithm.run_simulation(triangulation.tds().clone());

        log::info!("Simulation Results:");
        log::info!(
            "  Acceptance rate: {:.2}%",
            results.acceptance_rate() * 100.0
        );
        log::info!("  Average action: {:.3}", results.average_action());

        Ok(results)
    } else {
        // Just return basic simulation results with the triangulation
        use cdt::metropolis::Measurement;
        use std::time::Duration;

        let initial_action = config.to_action_config().calculate_action(
            u32::try_from(triangulation.vertex_count()).unwrap_or_default(),
            u32::try_from(triangulation.edge_count()).unwrap_or_default(),
            u32::try_from(triangulation.triangle_count()).unwrap_or_default(),
        );

        Ok(SimulationResults {
            config: config.to_metropolis_config(),
            action_config: config.to_action_config(),
            steps: vec![],
            measurements: vec![Measurement {
                step: 0,
                action: initial_action,
                vertices: u32::try_from(triangulation.vertex_count()).unwrap_or_default(),
                edges: u32::try_from(triangulation.edge_count()).unwrap_or_default(),
                triangles: u32::try_from(triangulation.triangle_count()).unwrap_or_default(),
            }],
            elapsed_time: Duration::from_millis(0),
            final_triangulation: triangulation.tds().clone(),
        })
    }
}

/// Runs the triangulation with the new trait-based backend system (RECOMMENDED).
///
/// This function uses the trait-based geometry backend system, which provides
/// better abstraction and testability compared to the legacy Tds-based approach.
///
/// # Arguments
///
/// * `config` - Configuration parameters for the triangulation/simulation
///
/// # Returns
///
/// A `SimulationResultsBackend` struct containing the results using the new backend.
///
/// # Errors
///
/// Returns [`CdtError::UnsupportedDimension`] if an unsupported dimension (not 2D) is specified.
/// Returns triangulation generation errors from the underlying triangulation creation.
pub fn run_with_backend(config: &Config) -> CdtResult<cdt::metropolis::SimulationResultsBackend> {
    let vertices = config.vertices;
    let timeslices = config.timeslices;

    if let Some(dim) = config.dimension
        && dim != 2
    {
        return Err(CdtError::UnsupportedDimension(dim.into()));
    }

    log::info!("Dimensionality: {}", config.dimension.unwrap_or(2));
    log::info!("Number of vertices: {vertices}");
    log::info!("Number of timeslices: {timeslices}");
    log::info!("Using new trait-based backend system");

    // Create initial triangulation with new backend
    let triangulation = CdtTriangulation::new_with_delaunay(vertices, timeslices, 2)?;

    log::info!(
        "Triangulation created with {} vertices, {} edges, {} faces",
        triangulation.vertex_count(),
        triangulation.edge_count(),
        triangulation.face_count()
    );

    if config.simulate {
        // Run full CDT simulation with new backend
        let metropolis_config = config.to_metropolis_config();
        let action_config = config.to_action_config();

        let mut algorithm = MetropolisAlgorithm::new(metropolis_config, action_config);
        let results = algorithm.run_simulation_with_backend(triangulation);

        log::info!("Simulation Results:");
        log::info!(
            "  Acceptance rate: {:.2}%",
            results.acceptance_rate() * 100.0
        );
        log::info!("  Average action: {:.3}", results.average_action());

        Ok(results)
    } else {
        // Just return basic simulation results with the triangulation
        use cdt::metropolis::Measurement;
        use std::time::Duration;

        let initial_action = config.to_action_config().calculate_action(
            u32::try_from(triangulation.vertex_count()).unwrap_or_default(),
            u32::try_from(triangulation.edge_count()).unwrap_or_default(),
            u32::try_from(triangulation.face_count()).unwrap_or_default(),
        );

        Ok(cdt::metropolis::SimulationResultsBackend {
            config: config.to_metropolis_config(),
            action_config: config.to_action_config(),
            steps: vec![],
            measurements: vec![Measurement {
                step: 0,
                action: initial_action,
                vertices: u32::try_from(triangulation.vertex_count()).unwrap_or_default(),
                edges: u32::try_from(triangulation.edge_count()).unwrap_or_default(),
                triangles: u32::try_from(triangulation.face_count()).unwrap_or_default(),
            }],
            elapsed_time: Duration::from_millis(0),
            triangulation,
        })
    }
}

#[cfg(test)]
mod lib_tests {
    use super::*;
    use approx::assert_relative_eq;

    fn create_test_config() -> Config {
        Config {
            dimension: Some(2),
            vertices: 32,
            timeslices: 3,
            temperature: 1.0,
            steps: 10,
            thermalization_steps: 5,
            measurement_frequency: 2,
            coupling_0: 1.0,
            coupling_2: 1.0,
            cosmological_constant: 0.1,
            simulate: false,
        }
    }

    #[test]
    #[allow(deprecated)]
    fn test_run() {
        let config = create_test_config();
        assert!(config.dimension.is_some());
        let results = run(&config).expect("Failed to run triangulation");
        assert!(!results.final_triangulation.cells().is_empty());
        assert!(!results.measurements.is_empty());
    }

    #[test]
    #[allow(deprecated)]
    fn triangulation_contains_triangles() {
        let config = create_test_config();
        let results = run(&config).expect("Failed to run triangulation");
        // Check that we have some triangles
        assert!(!results.final_triangulation.cells().is_empty());
    }

    #[test]
    fn test_config_conversions() {
        let config = create_test_config();

        let metropolis_config = config.to_metropolis_config();
        assert_relative_eq!(metropolis_config.temperature, 1.0);
        assert_eq!(metropolis_config.steps, 10);

        let action_config = config.to_action_config();
        assert_relative_eq!(action_config.coupling_0, 1.0);
        assert_relative_eq!(action_config.coupling_2, 1.0);
        assert_relative_eq!(action_config.cosmological_constant, 0.1);
    }
}
