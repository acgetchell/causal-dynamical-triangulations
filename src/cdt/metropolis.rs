//! Metropolis-Hastings algorithm for Causal Dynamical Triangulations.
//!
//! This module implements the Monte Carlo sampling algorithm used to sample
//! triangulation configurations according to the CDT path integral measure.

use crate::cdt::action::ActionConfig;
use crate::cdt::ergodic_moves::{ErgodicsSystem, MoveType};
use num_traits::cast::NumCast;
use std::time::Instant;

// Test utilities are now handled through backend-agnostic CdtTriangulation::new

/// Configuration for the Metropolis-Hastings algorithm.
#[derive(Debug, Clone)]
pub struct MetropolisConfig {
    /// Temperature parameter (1/β)
    pub temperature: f64,
    /// Number of Monte Carlo steps to perform
    pub steps: u32,
    /// Number of thermalization steps before measurements
    pub thermalization_steps: u32,
    /// Frequency of measurements (take measurement every N steps)
    pub measurement_frequency: u32,
}

impl Default for MetropolisConfig {
    /// Default Metropolis configuration for 2D CDT.
    fn default() -> Self {
        Self {
            temperature: 1.0,
            steps: 1000,
            thermalization_steps: 100,
            measurement_frequency: 10,
        }
    }
}

impl MetropolisConfig {
    /// Creates a new Metropolis configuration.
    #[must_use]
    pub const fn new(
        temperature: f64,
        steps: u32,
        thermalization_steps: u32,
        measurement_frequency: u32,
    ) -> Self {
        Self {
            temperature,
            steps,
            thermalization_steps,
            measurement_frequency,
        }
    }

    /// Returns the inverse temperature (β = 1/T).
    #[must_use]
    pub fn beta(&self) -> f64 {
        1.0 / self.temperature
    }
}

/// Result of a Monte Carlo step.
#[derive(Debug, Clone)]
pub struct MonteCarloStep {
    /// Step number
    pub step: u32,
    /// Move type attempted
    pub move_type: MoveType,
    /// Whether the move was accepted
    pub accepted: bool,
    /// Action before the move
    pub action_before: f64,
    /// Action after the move (if accepted)
    pub action_after: Option<f64>,
    /// Change in action (ΔS)
    pub delta_action: Option<f64>,
}

/// Measurement data collected during simulation.
#[derive(Debug, Clone)]
pub struct Measurement {
    /// Monte Carlo step when measurement was taken
    pub step: u32,
    /// Current action value
    pub action: f64,
    /// Number of vertices
    pub vertices: u32,
    /// Number of edges
    pub edges: u32,
    /// Number of triangles
    pub triangles: u32,
}

/// Metropolis-Hastings algorithm implementation for CDT.
///
/// This implementation works with both the legacy Tds-based approach
/// and the new trait-based geometry backends.
pub struct MetropolisAlgorithm {
    /// Algorithm configuration
    config: MetropolisConfig,
    /// Action calculation configuration
    action_config: ActionConfig,
    /// Ergodic moves system
    ergodics: ErgodicsSystem,
}

impl MetropolisAlgorithm {
    /// Creates a new Metropolis algorithm instance.
    #[must_use]
    pub fn new(config: MetropolisConfig, action_config: ActionConfig) -> Self {
        Self {
            config,
            action_config,
            ergodics: ErgodicsSystem::new(),
        }
    }

    /// Run the Monte Carlo simulation.
    ///
    /// This runs the Metropolis-Hastings algorithm on the given triangulation.
    pub fn run(
        &mut self,
        triangulation: crate::geometry::CdtTriangulation2D,
    ) -> SimulationResultsBackend {
        use crate::geometry::traits::TriangulationQuery;

        let start_time = Instant::now();
        let mut steps = Vec::new();
        let mut measurements = Vec::new();

        log::info!("Starting Metropolis-Hastings simulation with new backend...");
        log::info!("Temperature: {}", self.config.temperature);
        log::info!("Total steps: {}", self.config.steps);
        log::info!("Thermalization steps: {}", self.config.thermalization_steps);

        // Calculate initial action
        let geometry = triangulation.geometry();
        let current_action = self.action_config.calculate_action(
            u32::try_from(geometry.vertex_count()).unwrap_or_default(),
            u32::try_from(geometry.edge_count()).unwrap_or_default(),
            u32::try_from(geometry.face_count()).unwrap_or_default(),
        );

        for step_num in 0..self.config.steps {
            // For now, just simulate the step without actual moves
            // TODO: Implement ergodic moves for trait-based backends
            let move_type = self.ergodics.select_random_move();

            let mc_step = MonteCarloStep {
                step: step_num,
                move_type,
                accepted: false,
                action_before: current_action,
                action_after: None,
                delta_action: None,
            };

            steps.push(mc_step);

            // Take measurement if needed
            if step_num % self.config.measurement_frequency == 0 {
                let measurement = Measurement {
                    step: step_num,
                    action: current_action,
                    vertices: u32::try_from(geometry.vertex_count()).unwrap_or_default(),
                    edges: u32::try_from(geometry.edge_count()).unwrap_or_default(),
                    triangles: u32::try_from(geometry.face_count()).unwrap_or_default(),
                };
                measurements.push(measurement);
            }

            // Progress reporting
            if step_num % 100 == 0 {
                log::debug!(
                    "Step {}/{}, Action: {:.3}",
                    step_num,
                    self.config.steps,
                    current_action
                );
            }
        }

        let elapsed_time = start_time.elapsed();
        log::info!("Simulation completed in {elapsed_time:.2?}");

        SimulationResultsBackend {
            config: self.config.clone(),
            action_config: self.action_config.clone(),
            steps,
            measurements,
            elapsed_time,
            triangulation,
        }
    }
}

/// Results from a simulation using the new backend system.
#[derive(Debug)]
pub struct SimulationResultsBackend {
    /// Configuration used for the simulation
    pub config: MetropolisConfig,
    /// Action configuration used
    pub action_config: ActionConfig,
    /// All Monte Carlo steps performed
    pub steps: Vec<MonteCarloStep>,
    /// Measurements taken during simulation
    pub measurements: Vec<Measurement>,
    /// Total simulation time
    pub elapsed_time: std::time::Duration,
    /// Final triangulation state
    pub triangulation: crate::geometry::CdtTriangulation2D,
}

impl SimulationResultsBackend {
    /// Calculates the acceptance rate for the simulation.
    #[must_use]
    pub fn acceptance_rate(&self) -> f64 {
        if self.steps.is_empty() {
            return 0.0;
        }

        let accepted_count = self.steps.iter().filter(|step| step.accepted).count();
        let total_count = self.steps.len();

        let accepted_f64 = NumCast::from(accepted_count).unwrap_or(0.0);
        let total_f64 = NumCast::from(total_count).unwrap_or(1.0);

        accepted_f64 / total_f64
    }

    /// Calculates the average action over all measurements.
    #[must_use]
    pub fn average_action(&self) -> f64 {
        if self.measurements.is_empty() {
            return 0.0;
        }

        let sum: f64 = self.measurements.iter().map(|m| m.action).sum();
        let count = self.measurements.len();

        let count_f64 = NumCast::from(count).unwrap_or(1.0);

        sum / count_f64
    }

    /// Returns measurements after thermalization.
    #[must_use]
    pub fn equilibrium_measurements(&self) -> Vec<&Measurement> {
        self.measurements
            .iter()
            .filter(|m| m.step >= self.config.thermalization_steps)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_metropolis_config() {
        let config = MetropolisConfig::new(2.0, 500, 50, 5);
        assert_relative_eq!(config.temperature, 2.0);
        assert_relative_eq!(config.beta(), 0.5);
        assert_eq!(config.steps, 500);
    }

    #[test]
    fn test_backend_vertex_and_edge_counting() {
        use crate::cdt::triangulation::CdtTriangulation;
        use crate::geometry::traits::TriangulationQuery;

        let triangulation =
            CdtTriangulation::from_random_points(5, 1, 2).expect("Failed to create triangulation");
        let geometry = triangulation.geometry();

        // Test that the backend-based counting methods work
        let edge_count = geometry.edge_count();
        let vertex_count = geometry.vertex_count();
        let triangle_count = geometry.face_count();

        // Basic sanity checks
        assert!(vertex_count > 0);
        assert!(triangle_count > 0);
        assert!(edge_count > 0);

        // For a valid 2D triangulation, verify Euler's formula: V - E + F = 1
        #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
        let euler_check = vertex_count as i32 - edge_count as i32 + triangle_count as i32;

        // For a planar graph with boundary (disk topology), Euler's formula gives:
        // χ = V - E + F = 1
        assert_eq!(
            euler_check, 1,
            "Euler's formula V - E + F = 1 failed for planar graph with boundary: {vertex_count} - {edge_count} + {triangle_count} = {euler_check} (expected 1)"
        );
    }

    #[test]
    fn test_action_calculation() {
        use crate::cdt::triangulation::CdtTriangulation;
        use crate::geometry::traits::TriangulationQuery;

        let triangulation =
            CdtTriangulation::from_random_points(5, 1, 2).expect("Failed to create triangulation");

        let config = MetropolisConfig::default();
        let action_config = ActionConfig::default();
        let _algorithm = MetropolisAlgorithm::new(config, action_config.clone());

        let geometry = triangulation.geometry();
        let action = action_config.calculate_action(
            u32::try_from(geometry.vertex_count()).unwrap_or_default(),
            u32::try_from(geometry.edge_count()).unwrap_or_default(),
            u32::try_from(geometry.face_count()).unwrap_or_default(),
        );

        // Since we're using a random triangulation, just verify it returns a finite value
        assert!(action.is_finite());
    }

    #[test]
    fn test_simulation_results() {
        use crate::cdt::triangulation::CdtTriangulation;

        let config = MetropolisConfig::default();
        let measurements = vec![
            Measurement {
                step: 0,
                action: 1.0,
                vertices: 3,
                edges: 3,
                triangles: 1,
            },
            Measurement {
                step: 10,
                action: 2.0,
                vertices: 4,
                edges: 5,
                triangles: 2,
            },
        ];

        let triangulation =
            CdtTriangulation::from_random_points(3, 1, 2).expect("Failed to create triangulation");

        let results = SimulationResultsBackend {
            config,
            action_config: ActionConfig::default(),
            steps: vec![],
            measurements,
            elapsed_time: std::time::Duration::from_millis(100),
            triangulation,
        };

        assert_relative_eq!(results.average_action(), 1.5);
    }
}
