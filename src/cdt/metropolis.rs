//! Metropolis-Hastings algorithm for Causal Dynamical Triangulations.
//!
//! This module implements the Monte Carlo sampling algorithm used to sample
//! triangulation configurations according to the CDT path integral measure.

use crate::cdt::action::ActionConfig;
use crate::cdt::ergodic_moves::{ErgodicsSystem, MoveResult, MoveType};
use crate::errors::CdtError;
use crate::util::generate_random_float;
use delaunay::core::Tds;
use num_traits::cast::NumCast;
use std::time::Instant;

#[cfg(test)]
use crate::triangulations::triangulation::generate_random_delaunay2;

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

/// Results from a complete Metropolis simulation.
#[derive(Debug)]
pub struct SimulationResults<T, VertexData, CellData, const D: usize>
where
    T: delaunay::geometry::CoordinateScalar,
    VertexData: delaunay::core::DataType,
    CellData: delaunay::core::DataType,
    [T; D]: serde::Serialize + for<'de> serde::Deserialize<'de>,
{
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
    pub final_triangulation: Tds<T, VertexData, CellData, D>,
}

impl<T, VertexData, CellData, const D: usize> SimulationResults<T, VertexData, CellData, D>
where
    T: delaunay::geometry::CoordinateScalar,
    VertexData: delaunay::core::DataType,
    CellData: delaunay::core::DataType,
    [T; D]: serde::Serialize + for<'de> serde::Deserialize<'de>,
{
    /// Calculates the acceptance rate for the simulation.
    #[must_use]
    pub fn acceptance_rate(&self) -> f64 {
        if self.steps.is_empty() {
            return 0.0;
        }

        let accepted_count = self.steps.iter().filter(|step| step.accepted).count();
        let total_count = self.steps.len();

        // Use NumCast for safe conversion without precision loss warnings
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

        // Use NumCast for safe conversion without precision loss warnings
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

    /// Runs the complete Monte Carlo simulation.
    pub fn run_simulation<T, VertexData, CellData, const D: usize>(
        &mut self,
        triangulation: Tds<T, VertexData, CellData, D>,
    ) -> SimulationResults<T, VertexData, CellData, D>
    where
        T: delaunay::geometry::CoordinateScalar,
        VertexData: delaunay::core::DataType,
        CellData: delaunay::core::DataType,
        [T; D]: serde::Serialize + for<'de> serde::Deserialize<'de>,
    {
        let start_time = Instant::now();
        let mut steps = Vec::new();
        let mut measurements = Vec::new();

        log::info!("Starting Metropolis-Hastings simulation...");
        log::info!("Temperature: {}", self.config.temperature);
        log::info!("Total steps: {}", self.config.steps);
        log::info!("Thermalization steps: {}", self.config.thermalization_steps);

        // Calculate initial action
        let mut current_action = self.calculate_triangulation_action(&triangulation);

        for step_num in 0..self.config.steps {
            // Perform Monte Carlo step
            let mc_step = self.monte_carlo_step(&triangulation, current_action, step_num);

            // Update current action if move was accepted
            if let Some(new_action) = mc_step.action_after {
                current_action = new_action;
            }

            steps.push(mc_step);

            // Take measurement if needed
            if step_num % self.config.measurement_frequency == 0 {
                let measurement = Measurement {
                    step: step_num,
                    action: current_action,
                    vertices: u32::try_from(triangulation.vertices().len()).unwrap_or_default(),
                    edges: Self::count_edges_from_tds(&triangulation),
                    triangles: u32::try_from(triangulation.cells().len()).unwrap_or_default(),
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

        SimulationResults {
            config: self.config.clone(),
            action_config: self.action_config.clone(),
            steps,
            measurements,
            elapsed_time,
            final_triangulation: triangulation,
        }
    }

    /// Performs a single Monte Carlo step.
    fn monte_carlo_step<T, VertexData, CellData, const D: usize>(
        &mut self,
        triangulation: &Tds<T, VertexData, CellData, D>,
        current_action: f64,
        step_num: u32,
    ) -> MonteCarloStep
    where
        T: delaunay::geometry::CoordinateScalar,
        VertexData: delaunay::core::DataType,
        CellData: delaunay::core::DataType,
        [T; D]: serde::Serialize + for<'de> serde::Deserialize<'de>,
    {
        // Select and attempt a random move
        let move_type = self.ergodics.select_random_move();

        // QUALITY ISSUE LOG: Incomplete ergodic moves implementation
        log::warn!(
            "QUALITY ISSUE: Ergodic move {move_type:?} attempted but not implemented for Tds"
        );
        log::warn!("This is a critical gap - CDT simulation cannot properly evolve triangulations");
        log::warn!(
            "Expected behavior: Move should modify triangulation and return Success/Rejection"
        );
        log::warn!(
            "Actual behavior: All moves automatically rejected due to missing implementation"
        );

        // For now, just return a placeholder since ergodic moves need to be adapted for Tds
        let move_result = MoveResult::Rejected(CdtError::ErgodicsFailure(
            "Tds-based moves not yet implemented".to_string(),
        ));

        let mut mc_step = MonteCarloStep {
            step: step_num,
            move_type,
            accepted: false,
            action_before: current_action,
            action_after: None,
            delta_action: None,
        };

        // If move was successfully applied, check Metropolis criterion
        if matches!(move_result, MoveResult::Success) {
            let new_action = self.calculate_triangulation_action(triangulation);
            let delta_action = new_action - current_action;

            mc_step.delta_action = Some(delta_action);

            // Metropolis acceptance criterion
            let accept_probability = if delta_action <= 0.0 {
                1.0
            } else {
                (-self.config.beta() * delta_action).exp()
            };

            if generate_random_float() < accept_probability {
                // Accept the move
                mc_step.accepted = true;
                mc_step.action_after = Some(new_action);
            } else {
                // Reject the move - revert to original state
                // Note: In current implementation, moves are simulated rather than
                // actually applied, so no reversal is needed. Future versions with
                // real Tds manipulation will need proper state management.
            }
        }

        mc_step
    }

    /// Calculates the action for the current triangulation.
    #[must_use]
    fn calculate_triangulation_action<T, VertexData, CellData, const D: usize>(
        &self,
        triangulation: &Tds<T, VertexData, CellData, D>,
    ) -> f64
    where
        T: delaunay::geometry::CoordinateScalar,
        VertexData: delaunay::core::DataType,
        CellData: delaunay::core::DataType,
        [T; D]: serde::Serialize + for<'de> serde::Deserialize<'de>,
    {
        let vertices = u32::try_from(triangulation.vertices().len()).unwrap_or_default();
        let edges = Self::count_edges_from_tds(triangulation);
        let triangles = u32::try_from(triangulation.cells().len()).unwrap_or_default();

        self.action_config
            .calculate_action(vertices, edges, triangles)
    }

    /// Counts the number of edges in the triangulation from a Tds.
    /// Uses the shared canonical implementation from the triangulation module.
    #[must_use]
    fn count_edges_from_tds<T, VertexData, CellData, const D: usize>(
        triangulation: &Tds<T, VertexData, CellData, D>,
    ) -> u32
    where
        T: delaunay::geometry::CoordinateScalar,
        VertexData: delaunay::core::DataType,
        CellData: delaunay::core::DataType,
        [T; D]: serde::Serialize + for<'de> serde::Deserialize<'de>,
    {
        use crate::triangulations::triangulation::count_edges_in_tds;

        let edge_count = count_edges_in_tds(triangulation);
        u32::try_from(edge_count).unwrap_or_default()
    }
    /// Run simulation with new trait-based `CdtTriangulation` (RECOMMENDED).
    ///
    /// This is the recommended interface that works with geometry backends.
    /// Provides better abstraction than the legacy `run_simulation()` method.
    pub fn run_simulation_with_backend(
        &mut self,
        triangulation: crate::cdt::triangulation::CdtTriangulation<
            crate::geometry::backends::delaunay::DelaunayBackend2D,
        >,
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
    pub triangulation: crate::cdt::triangulation::CdtTriangulation<
        crate::geometry::backends::delaunay::DelaunayBackend2D,
    >,
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
    fn test_tds_vertex_and_edge_counting() {
        let triangulation = generate_random_delaunay2(5, (0.0, 10.0));

        // Test that the Tds-based counting methods work
        let edge_count = MetropolisAlgorithm::count_edges_from_tds(&triangulation);
        let vertex_count = triangulation.vertices().len();
        let triangle_count = triangulation.cells().len();

        // Basic sanity checks
        assert!(vertex_count > 0);
        assert!(triangle_count > 0);
        assert!(edge_count > 0);

        // For a valid 2D triangulation, verify Euler's formula: T - E + V = 2
        // Rearranged: E = V + T - 2
        #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
        let euler_check = triangle_count as i32 - edge_count as i32 + vertex_count as i32;

        // For a planar graph with boundary (not a closed surface), Euler's formula gives:
        // χ = V - E + F = 1 (for a disk topology)
        // Rearranged as: F - E + V = 1, or in our case: T - E + V = 1
        assert_eq!(
            euler_check, 1,
            "Euler's formula T - E + V = 1 failed for planar graph with boundary: {triangle_count} - {edge_count} + {vertex_count} = {euler_check} (expected 1)"
        );
    }

    #[test]
    fn test_action_calculation() {
        let triangulation = generate_random_delaunay2(3, (0.0, 10.0));

        let config = MetropolisConfig::default();
        let action_config = ActionConfig::default();
        let algorithm = MetropolisAlgorithm::new(config, action_config);

        let action = algorithm.calculate_triangulation_action(&triangulation);

        // Since we're using a random triangulation, just verify it returns a finite value
        assert!(action.is_finite());
    }

    #[test]
    fn test_simulation_results() {
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

        let final_triangulation = generate_random_delaunay2(3, (0.0, 10.0));

        let results = SimulationResults {
            config,
            action_config: ActionConfig::default(),
            steps: vec![],
            measurements,
            elapsed_time: std::time::Duration::from_millis(100),
            final_triangulation,
        };

        assert_relative_eq!(results.average_action(), 1.5);
    }
}
