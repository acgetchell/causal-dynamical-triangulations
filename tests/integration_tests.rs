//! Comprehensive integration tests for CDT-RS.
//!
//! This module contains integration tests that verify the complete CDT simulation
//! workflows, topology preservation, error handling, and consistency between components.

use causal_dynamical_triangulations::cdt::action::ActionConfig;
use causal_dynamical_triangulations::cdt::metropolis::{MetropolisAlgorithm, MetropolisConfig};
use causal_dynamical_triangulations::triangulations::triangulation::{
    CausalTriangulation2D, count_edges_in_tds, generate_random_delaunay2,
};

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_complete_cdt_simulation_workflow() {
        // Test full CDT simulation pipeline
        let triangulation =
            CausalTriangulation2D::new(8, 2, 2).expect("Failed to create initial triangulation");

        let config = MetropolisConfig::new(1.0, 50, 10, 5);
        let action_config = ActionConfig::default();
        let mut algorithm = MetropolisAlgorithm::new(config, action_config);

        // Run simulation
        let results = algorithm.run_simulation(triangulation.tds().clone());

        // Verify results
        assert!(!results.steps.is_empty(), "Simulation should produce steps");
        assert!(
            !results.measurements.is_empty(),
            "Simulation should produce measurements"
        );
        assert!(
            results.acceptance_rate() >= 0.0,
            "Acceptance rate should be non-negative"
        );
        assert!(
            results.acceptance_rate() <= 1.0,
            "Acceptance rate should not exceed 1.0"
        );
        assert!(
            results.average_action().is_finite(),
            "Average action should be finite"
        );
        assert!(
            results.elapsed_time.as_millis() > 0,
            "Simulation should take measurable time"
        );
    }

    #[test]
    fn test_edge_counting_consistency() {
        // Test that both edge counting methods produce identical results
        let tds = generate_random_delaunay2(7, (0.0, 10.0));
        let triangulation = CausalTriangulation2D::from_tds(tds.clone(), 3, 2);

        let method1_count = triangulation.edge_count();
        let method2_count = count_edges_in_tds(&tds);

        assert_eq!(
            method1_count, method2_count,
            "Both edge counting methods should produce identical results"
        );
    }

    #[test]
    fn test_topology_invariants() {
        let triangulation =
            CausalTriangulation2D::new(6, 1, 2).expect("Failed to create triangulation");

        let v = i32::try_from(triangulation.vertex_count()).unwrap_or(i32::MAX);
        let e = i32::try_from(triangulation.edge_count()).unwrap_or(i32::MAX);
        let t = i32::try_from(triangulation.triangle_count()).unwrap_or(i32::MAX);

        // Verify Euler's formula for planar graphs: V - E + T = 1
        assert_eq!(v - e + t, 1, "Euler's formula V - E + T = 1 must hold");

        // Verify all counts are positive
        assert!(v > 0, "Must have positive vertex count");
        assert!(e > 0, "Must have positive edge count");
        assert!(t > 0, "Must have positive triangle count");
    }

    #[test]
    fn test_enhanced_caching_behavior() {
        let mut triangulation =
            CausalTriangulation2D::new(5, 1, 2).expect("Failed to create triangulation");

        // Test cache population
        let initial_count = triangulation.edge_count();
        let cached_count = triangulation.edge_count(); // Should use cache
        assert_eq!(initial_count, cached_count);

        // Test that cache is preserved when no modifications occur
        {
            let _wrapper = triangulation.tds_mut();
            // Don't call mark_modified() - cache should be preserved
        }

        let count_after_wrapper = triangulation.edge_count();
        assert_eq!(
            initial_count, count_after_wrapper,
            "Cache should be preserved without modifications"
        );

        // Test cache invalidation when modifications are explicitly marked
        {
            let wrapper = triangulation.tds_mut();
            wrapper.mark_modified(); // Explicitly mark as modified
        }

        let recalculated_count = triangulation.edge_count();
        assert_eq!(
            initial_count, recalculated_count,
            "Results should be consistent after cache invalidation"
        );
    }

    #[test]
    fn test_error_handling_robustness() {
        // Test parameter validation with enhanced error context
        let result = CausalTriangulation2D::new(2, 1, 2);
        assert!(result.is_err(), "Should reject < 3 vertices");

        let result = CausalTriangulation2D::new(5, 0, 2);
        assert!(result.is_err(), "Should reject 0 timeslices");

        let result = CausalTriangulation2D::new(5, 1, 3);
        assert!(result.is_err(), "Should reject non-2D");

        // Test successful minimum case
        let min_triangulation = CausalTriangulation2D::new(3, 1, 2);
        assert!(
            min_triangulation.is_ok(),
            "Minimum valid parameters should succeed"
        );
    }

    #[test]
    fn test_action_calculation_consistency() {
        let triangulation =
            CausalTriangulation2D::new(4, 1, 2).expect("Failed to create triangulation");

        let config = ActionConfig::default();
        let vertices = u32::try_from(triangulation.vertex_count()).unwrap_or_default();
        let edges = u32::try_from(triangulation.edge_count()).unwrap_or_default();
        let triangles = u32::try_from(triangulation.triangle_count()).unwrap_or_default();

        let action = config.calculate_action(vertices, edges, triangles);

        // Action should be finite and non-NaN
        assert!(
            action.is_finite(),
            "Action calculation must produce finite results"
        );

        // For default config (κ₀=1.0, κ₂=1.0, λ=0.1): S = -V - T + 0.1*E
        let expected = 0.1f64.mul_add(
            f64::from(edges),
            -f64::from(vertices) - f64::from(triangles),
        );
        assert!(
            (action - expected).abs() < f64::EPSILON,
            "Action formula should match expected calculation"
        );
    }

    #[test]
    fn test_triangulation_generation_error_context() {
        // Test enhanced error context for triangulation generation
        use causal_dynamical_triangulations::triangulations::triangulation::try_generate_random_delaunay2_with_context;

        // Test invalid vertex count
        let result = try_generate_random_delaunay2_with_context(2, (0.0, 10.0));
        assert!(result.is_err(), "Should reject insufficient vertices");

        // Test invalid coordinate range
        let result = try_generate_random_delaunay2_with_context(5, (10.0, 0.0));
        assert!(result.is_err(), "Should reject invalid coordinate range");

        // Test successful generation
        let result = try_generate_random_delaunay2_with_context(4, (0.0, 10.0));
        assert!(result.is_ok(), "Should succeed with valid parameters");
    }

    #[test]
    fn test_simulation_reproducibility() {
        // Test that simulations with same parameters produce consistent results structure
        let triangulation1 =
            CausalTriangulation2D::new(5, 1, 2).expect("Failed to create first triangulation");
        let triangulation2 =
            CausalTriangulation2D::new(5, 1, 2).expect("Failed to create second triangulation");

        let config = MetropolisConfig::new(1.0, 10, 2, 2);
        let action_config = ActionConfig::default();

        let mut algorithm1 = MetropolisAlgorithm::new(config.clone(), action_config.clone());
        let mut algorithm2 = MetropolisAlgorithm::new(config, action_config);

        let results1 = algorithm1.run_simulation(triangulation1.tds().clone());
        let results2 = algorithm2.run_simulation(triangulation2.tds().clone());

        // Results should have same structure (though values may differ due to randomness)
        assert_eq!(
            results1.steps.len(),
            results2.steps.len(),
            "Should have same number of steps"
        );
        assert_eq!(
            results1.measurements.len(),
            results2.measurements.len(),
            "Should have same number of measurements"
        );

        // Both should produce valid results
        assert!(results1.acceptance_rate().is_finite() && results1.acceptance_rate() >= 0.0);
        assert!(results2.acceptance_rate().is_finite() && results2.acceptance_rate() >= 0.0);
    }

    #[test]
    fn test_memory_efficiency() {
        // Test that large triangulations can be created and processed efficiently
        let triangulation =
            CausalTriangulation2D::new(20, 1, 2).expect("Failed to create large triangulation");

        // Verify reasonable scaling of components
        let vertices = triangulation.vertex_count();
        let edges = triangulation.edge_count();
        let triangles = triangulation.triangle_count();

        assert!(vertices == 20, "Should have requested number of vertices");
        assert!(edges > vertices, "Should have more edges than vertices");
        assert!(triangles > 0, "Should have positive triangle count");

        // Test that edge counting is efficient (doesn't hang)
        let start = std::time::Instant::now();
        let _ = triangulation.edge_count();
        let elapsed = start.elapsed();

        assert!(
            elapsed.as_millis() < 1000,
            "Edge counting should complete quickly"
        );
    }
}
