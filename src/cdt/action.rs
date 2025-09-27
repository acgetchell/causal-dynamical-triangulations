//! 2D Regge Action calculation for Causal Dynamical Triangulations.
//!
//! This module implements the discrete Einstein-Hilbert action used in CDT,
//! which is based on the Regge calculus formulation of general relativity.

/// Calculates the 2D Regge Action for a given triangulation.
///
/// The 2D Regge Action in CDT is given by:
/// S = -κ₀ N₀ - κ₂ N₂ + λ N₁
/// where:
/// - N₀ = number of vertices (0-simplices)
/// - N₁ = number of edges (1-simplices)
/// - N₂ = number of triangles (2-simplices)
/// - κ₀, κ₂ = coupling constants
/// - λ = cosmological constant
///
/// # Arguments
///
/// * `vertices` - Number of vertices in the triangulation
/// * `edges` - Number of edges in the triangulation
/// * `triangles` - Number of triangles in the triangulation
/// * `coupling_0` - Coupling constant κ₀ for vertices
/// * `coupling_2` - Coupling constant κ₂ for triangles
/// * `cosmological_constant` - Cosmological constant λ
///
/// # Returns
///
/// The calculated Regge Action value
#[must_use]
pub fn calculate_regge_action_2d(
    vertices: u32,
    edges: u32,
    triangles: u32,
    coupling_0: f64,
    coupling_2: f64,
    cosmological_constant: f64,
) -> f64 {
    let n_0 = f64::from(vertices);
    let n_1 = f64::from(edges);
    let n_2 = f64::from(triangles);

    cosmological_constant.mul_add(n_1, (-coupling_0).mul_add(n_0, -(coupling_2 * n_2)))
}

/// Configuration for CDT action parameters.
#[derive(Debug, Clone)]
pub struct ActionConfig {
    /// Coupling constant for vertices (κ₀)
    pub coupling_0: f64,
    /// Coupling constant for triangles (κ₂)
    pub coupling_2: f64,
    /// Cosmological constant (λ)
    pub cosmological_constant: f64,
}

impl Default for ActionConfig {
    /// Default CDT action parameters for 2D simulations.
    fn default() -> Self {
        Self {
            coupling_0: 1.0,
            coupling_2: 1.0,
            cosmological_constant: 0.1,
        }
    }
}

impl ActionConfig {
    /// Creates a new action configuration.
    #[must_use]
    pub const fn new(coupling_0: f64, coupling_2: f64, cosmological_constant: f64) -> Self {
        Self {
            coupling_0,
            coupling_2,
            cosmological_constant,
        }
    }

    /// Calculates the action for given simplex counts.
    #[must_use]
    pub fn calculate_action(&self, vertices: u32, edges: u32, triangles: u32) -> f64 {
        calculate_regge_action_2d(
            vertices,
            edges,
            triangles,
            self.coupling_0,
            self.coupling_2,
            self.cosmological_constant,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_regge_action_calculation() {
        let vertices = 10;
        let edges = 20;
        let triangles = 15;
        let coupling_0 = 1.0;
        let coupling_2 = 1.0;
        let cosmological_constant = 0.1;

        let action = calculate_regge_action_2d(
            vertices,
            edges,
            triangles,
            coupling_0,
            coupling_2,
            cosmological_constant,
        );

        // Expected: -1.0 * 10 - 1.0 * 15 + 0.1 * 20 = -10 - 15 + 2 = -23
        let expected = -23.0;
        assert_relative_eq!(action, expected);
    }

    #[test]
    fn test_action_config_default() {
        let config = ActionConfig::default();
        assert_relative_eq!(config.coupling_0, 1.0);
        assert_relative_eq!(config.coupling_2, 1.0);
        assert_relative_eq!(config.cosmological_constant, 0.1);
    }

    #[test]
    fn test_action_config_calculate() {
        let config = ActionConfig::new(2.0, 1.5, 0.2);
        let action = config.calculate_action(5, 10, 8);

        // Expected: -2.0 * 5 - 1.5 * 8 + 0.2 * 10 = -10 - 12 + 2 = -20
        let expected = -20.0;
        assert_relative_eq!(action, expected);
    }
}
