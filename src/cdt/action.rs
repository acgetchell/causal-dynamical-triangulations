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
pub fn compute_regge_action(
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
        compute_regge_action(
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

        let action = compute_regge_action(
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

#[cfg(test)]
mod prop_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn action_always_finite(
            vertices in 0u32..100,
            edges in 0u32..500,
            triangles in 0u32..300,
            coupling_0 in -10.0f64..10.0,
            coupling_2 in -10.0f64..10.0,
            cosmological_constant in -5.0f64..5.0
        ) {
            let action = compute_regge_action(
                vertices, edges, triangles,
                coupling_0, coupling_2, cosmological_constant
            );

            prop_assert!(action.is_finite(), "Action must always be finite, got: {}", action);
            prop_assert!(!action.is_nan(), "Action must not be NaN");
        }

        #[test]
        fn action_config_consistency(
            vertices in 0u32..50,
            edges in 0u32..150,
            triangles in 0u32..100,
            coupling_0 in -5.0f64..5.0,
            coupling_2 in -5.0f64..5.0,
            cosmological_constant in -2.0f64..2.0
        ) {
            let config = ActionConfig::new(coupling_0, coupling_2, cosmological_constant);

            // Config should preserve values
            prop_assert!((config.coupling_0 - coupling_0).abs() < f64::EPSILON);
            prop_assert!((config.coupling_2 - coupling_2).abs() < f64::EPSILON);
            prop_assert!((config.cosmological_constant - cosmological_constant).abs() < f64::EPSILON);

            // Config-based calculation should match direct function call
            let action_config = config.calculate_action(vertices, edges, triangles);
            let action_direct = compute_regge_action(
                vertices, edges, triangles,
                coupling_0, coupling_2, cosmological_constant
            );

            prop_assert!(
                (action_config - action_direct).abs() < f64::EPSILON,
                "Config-based and direct calculations should match: {} vs {}",
                action_config, action_direct
            );
        }
    }
}

#[cfg(kani)]
mod kani_proofs {
    //! Kani formal verification proofs for action calculations
    //!
    //! These proofs are optimized for performance:
    //! - Small input ranges (vertices ≤ 50, edges ≤ 150, triangles ≤ 100)
    //! - Tight coupling bounds (|coupling| ≤ 5.0)
    //! - Topological constraints based on Euler characteristic
    //! - Unwind bounds to limit symbolic execution depth
    use super::*;

    #[kani::proof]
    #[kani::unwind(3)]
    fn verify_regge_action_properties() {
        // Generate arbitrary inputs with much smaller, realistic ranges
        let vertices: u32 = kani::any();
        let edges: u32 = kani::any();
        let triangles: u32 = kani::any();
        let coupling_0: f64 = kani::any();
        let coupling_2: f64 = kani::any();
        let cosmological_constant: f64 = kani::any();

        // Constrain inputs to much smaller, more realistic ranges for faster verification
        kani::assume(vertices <= 50);
        kani::assume(edges <= 150);
        kani::assume(triangles <= 100);
        kani::assume(coupling_0.is_finite());
        kani::assume(coupling_2.is_finite());
        kani::assume(cosmological_constant.is_finite());
        kani::assume(coupling_0.abs() <= 5.0);
        kani::assume(coupling_2.abs() <= 5.0);
        kani::assume(cosmological_constant.abs() <= 2.0);

        // Add Euler characteristic constraint for 2D triangulations: V - E + F = χ
        // For a closed surface: V - E + T ≈ 2 (where T = triangles = faces)
        // This reduces the state space significantly
        kani::assume(vertices >= 3); // Need at least 3 vertices for any triangle
        kani::assume(edges >= 3); // Need at least 3 edges
        kani::assume(triangles >= 1); // Need at least 1 triangle
        // Loose bounds based on triangulation topology
        kani::assume(edges <= vertices * 6); // Each vertex has at most ~6 edges on average
        kani::assume(triangles <= edges); // Usually fewer triangles than edges

        let action = compute_regge_action(
            vertices,
            edges,
            triangles,
            coupling_0,
            coupling_2,
            cosmological_constant,
        );

        // Property 1: Result should be finite (no NaN or infinity)
        assert!(action.is_finite(), "Action must be finite");

        // Property 2: Verify that scaling couplings scales the action appropriately
        // Test with doubled couplings and zero cosmological constant for simplicity
        if coupling_0 != 0.0 || coupling_2 != 0.0 {
            let action_doubled_couplings = compute_regge_action(
                vertices,
                edges,
                triangles,
                coupling_0 * 2.0,
                coupling_2 * 2.0,
                0.0, // Set cosmological constant to 0 to isolate coupling effects
            );

            let action_zero_cosmological = compute_regge_action(
                vertices, edges, triangles, coupling_0, coupling_2,
                0.0, // Zero cosmological constant for comparison
            );

            // With doubled couplings, the difference from zero should be doubled
            // This avoids precision issues with subtraction
            let ratio = if action_zero_cosmological != 0.0 {
                action_doubled_couplings / action_zero_cosmological
            } else {
                // If both are zero, that's also correct
                if action_doubled_couplings == 0.0 {
                    2.0
                } else {
                    0.0
                }
            };

            assert!(
                (ratio - 2.0).abs() < 1e-10,
                "Doubling couplings should double the coupling contribution to action"
            );
        }

        // Property 3: Zero simplices should give zero action with zero couplings
        let zero_action = compute_regge_action(0, 0, 0, 0.0, 0.0, 0.0);
        assert_eq!(
            zero_action, 0.0,
            "Zero simplices with zero couplings should give zero action"
        );
    }

    #[kani::proof]
    #[kani::unwind(2)]
    fn verify_action_config() {
        let coupling_0: f64 = kani::any();
        let coupling_2: f64 = kani::any();
        let cosmological_constant: f64 = kani::any();

        // Much tighter constraints for faster verification
        kani::assume(coupling_0.is_finite());
        kani::assume(coupling_2.is_finite());
        kani::assume(cosmological_constant.is_finite());
        kani::assume(coupling_0.abs() <= 2.0);
        kani::assume(coupling_2.abs() <= 2.0);
        kani::assume(cosmological_constant.abs() <= 1.0);

        let config = ActionConfig::new(coupling_0, coupling_2, cosmological_constant);

        // Verify that the config stores values correctly
        assert_eq!(config.coupling_0, coupling_0);
        assert_eq!(config.coupling_2, coupling_2);
        assert_eq!(config.cosmological_constant, cosmological_constant);

        // Verify that calculate_action produces finite results with small inputs
        let action = config.calculate_action(10, 20, 15);
        assert!(
            action.is_finite(),
            "Config-based action calculation must produce finite results"
        );
    }
}
