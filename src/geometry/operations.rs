//! High-level triangulation operations.
//!
//! This module provides common operations that work across different
//! geometry backends.

use super::traits::TriangulationQuery;

/// Common utility operations for triangulations
pub trait TriangulationOps: TriangulationQuery {
    /// Check if the triangulation satisfies Delaunay property (if applicable)
    fn is_delaunay(&self) -> bool {
        // TODO: Implement Delaunay property checking
        // This would need to check the circumcircle property for all triangles
        true
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
