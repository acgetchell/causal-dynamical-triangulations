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
