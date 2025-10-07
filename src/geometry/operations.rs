//! High-level triangulation operations.
//!
//! This module provides common operations that work across different
//! geometry backends.

use super::traits::TriangulationQuery;

/// Common utility operations for triangulations
pub trait TriangulationOps: TriangulationQuery {
    /// Check if the triangulation satisfies Delaunay property (if applicable)
    fn is_delaunay(&self) -> bool {
        // Delegate to the backend's validation method
        // For Delaunay backends with appropriate trait bounds, this checks the
        // circumcircle property. For other backends, it checks basic validity.
        self.is_valid()
    }

    /// Compute the convex hull of the triangulation
    fn convex_hull(&self) -> Vec<Self::VertexHandle> {
        // TODO: Implement convex hull computation
        Vec::new()
    }

    /// Find all boundary edges
    fn boundary_edges(&self) -> Vec<Self::EdgeHandle> {
        // TODO: Implement boundary detection
        Vec::new()
    }
}

// Blanket implementation for all types that implement TriangulationQuery
impl<T: TriangulationQuery> TriangulationOps for T {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::backends::mock::MockBackend;

    #[test]
    fn test_is_delaunay_delegates_to_is_valid() {
        let backend = MockBackend::create_triangle();

        // For our mock backend, is_delaunay should delegate to is_valid
        // which returns true for a basic triangle
        assert!(backend.is_delaunay());
    }

    #[test]
    fn test_convex_hull_placeholder() {
        let backend = MockBackend::create_triangle();

        // Current implementation returns empty vector
        let hull = backend.convex_hull();
        assert!(hull.is_empty());
    }

    #[test]
    fn test_boundary_edges_placeholder() {
        let backend = MockBackend::create_triangle();

        // Current implementation returns empty vector
        let boundary = backend.boundary_edges();
        assert!(boundary.is_empty());
    }

    #[test]
    fn test_triangulation_ops_trait_available() {
        let backend = MockBackend::create_triangle();

        // Test that the blanket implementation provides the trait methods
        let _is_delaunay = backend.is_delaunay();
        let _hull = backend.convex_hull();
        let _boundary = backend.boundary_edges();

        // If we get here without compilation errors, the trait is working
    }
}
