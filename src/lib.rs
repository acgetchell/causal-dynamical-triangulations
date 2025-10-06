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
//! use causal_dynamical_triangulations::{CdtConfig, run_simulation};
//! // CdtConfig requires configuration, so this is marked no_run
//! ```

// Module declarations (avoiding mod.rs files)
/// Configuration management for CDT simulations.
pub mod config;

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

    // Type aliases for common backend combinations
    /// 2D Delaunay backend with f64 coordinates (most common configuration)
    pub type DelaunayBackend2D = backends::delaunay::DelaunayBackend<f64, i32, i32, 2>;

    /// Default backend type for 2D CDT simulations
    pub type DefaultBackend = DelaunayBackend2D;

    /// Convenient alias for CDT triangulations using the default backend
    pub type CdtTriangulation2D = crate::cdt::triangulation::CdtTriangulation<DefaultBackend>;
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

// Re-exports for convenience
pub use cdt::action::{ActionConfig, compute_regge_action};
pub use cdt::ergodic_moves::{ErgodicsSystem, MoveResult, MoveType};
pub use cdt::metropolis::{MetropolisAlgorithm, MetropolisConfig, SimulationResultsBackend};
pub use config::{CdtConfig, TestConfig};
pub use errors::{CdtError, CdtResult};

// Trait-based triangulation (recommended)
pub use cdt::triangulation::CdtTriangulation;

/// Runs a CDT simulation with the specified configuration.
///
/// This function uses the trait-based geometry backend system, which provides
/// better abstraction and testability compared to legacy approaches.
///
/// # Arguments
///
/// * `config` - Configuration parameters for the triangulation/simulation
///
/// # Returns
///
/// A `SimulationResults` struct containing the results of the simulation.
///
/// # Errors
///
/// Returns [`CdtError::UnsupportedDimension`] if an unsupported dimension (not 2D) is specified.
/// Returns triangulation generation errors from the underlying triangulation creation.
pub fn run_simulation(config: &CdtConfig) -> CdtResult<cdt::metropolis::SimulationResultsBackend> {
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
    log::info!("Using trait-based backend system");

    // Create initial triangulation
    let triangulation = CdtTriangulation::from_random_points(vertices, timeslices, 2)?;

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
        let results = algorithm.run(triangulation);

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

    fn create_test_config() -> CdtConfig {
        CdtConfig {
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
    fn test_run_simulation() {
        let config = create_test_config();
        assert!(config.dimension.is_some());
        let results = run_simulation(&config).expect("Failed to run triangulation");
        assert!(results.triangulation.face_count() > 0);
        assert!(!results.measurements.is_empty());
    }

    #[test]
    fn triangulation_contains_triangles() {
        let config = create_test_config();
        let results = run_simulation(&config).expect("Failed to run triangulation");
        // Check that we have some triangles
        assert!(results.triangulation.face_count() > 0);
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
