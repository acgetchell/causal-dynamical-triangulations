//! Triangulation module for CDT.
//!
//! This module provides the core triangulation functionality for
//! Causal Dynamical Triangulations, including integration with
//! the delaunay crate and CDT-specific data structures.

use delaunay::geometry::util::generate_random_triangulation;

/// Main triangulation structure for CDT simulations.
///
/// This struct wraps a Tds (Triangulated Data Structure) from the delaunay crate
/// and adds CDT-specific metadata like time slices.
#[derive(Debug, Clone)]
pub struct CausalTriangulation<T, VertexData, CellData, const D: usize>
where
    T: delaunay::geometry::CoordinateScalar,
    VertexData: delaunay::core::DataType,
    CellData: delaunay::core::DataType,
    [T; D]: serde::Serialize + for<'de> serde::Deserialize<'de>,
{
    /// The underlying triangulation data structure from delaunay crate
    pub tds: delaunay::core::Tds<T, VertexData, CellData, D>,
    /// Number of time slices in the foliation
    pub time_slices: u32,
    /// Dimension of the triangulation
    pub dimension: u32,
}

/// Type alias for 2D triangulations with f64 coordinates
pub type CausalTriangulation2D = CausalTriangulation<f64, i32, i32, 2>;

impl<T, VertexData, CellData, const D: usize> CausalTriangulation<T, VertexData, CellData, D>
where
    T: delaunay::geometry::CoordinateScalar,
    VertexData: delaunay::core::DataType,
    CellData: delaunay::core::DataType,
    [T; D]: serde::Serialize + for<'de> serde::Deserialize<'de>,
{
    /// Creates a new causal triangulation from an existing Tds.
    #[must_use]
    pub const fn from_tds(
        tds: delaunay::core::Tds<T, VertexData, CellData, D>,
        time_slices: u32,
        dimension: u32,
    ) -> Self {
        Self {
            tds,
            time_slices,
            dimension,
        }
    }

    /// Returns the number of triangles in the triangulation.
    #[must_use]
    pub fn triangle_count(&self) -> usize {
        self.tds.cells().len()
    }

    /// Returns the number of unique vertices in the triangulation.
    #[must_use]
    pub fn vertex_count(&self) -> usize {
        self.tds.vertices().len()
    }

    /// Returns the number of edges in the triangulation.
    ///
    /// Since the delaunay crate doesn't provide direct edge access in all versions,
    /// we calculate this using Euler's formula: E = V + F - 2 for a planar graph.
    #[must_use]
    pub fn edge_count(&self) -> usize {
        let vertices = self.vertex_count();
        let faces = self.triangle_count();
        if vertices >= 2 && faces > 0 {
            vertices + faces - 2
        } else {
            0
        }
    }

    /// Prints a summary of the triangulation.
    pub fn print_summary(&self) {
        println!("Causal Triangulation Summary:");
        println!("  Dimension: {}", self.dimension);
        println!("  Time slices: {}", self.time_slices);
        println!("  Vertices: {}", self.vertex_count());
        println!("  Edges: {}", self.edge_count());
        println!("  Triangles: {}", self.triangle_count());
    }

    /// Returns a reference to the underlying Tds.
    #[must_use]
    pub const fn tds(&self) -> &delaunay::core::Tds<T, VertexData, CellData, D> {
        &self.tds
    }

    /// Returns a mutable reference to the underlying Tds.
    pub const fn tds_mut(&mut self) -> &mut delaunay::core::Tds<T, VertexData, CellData, D> {
        &mut self.tds
    }
}

// Specific implementation for 2D triangulations
impl CausalTriangulation2D {
    /// Creates a new 2D causal triangulation.
    ///
    /// # Arguments
    ///
    /// * `vertices` - Number of vertices in the triangulation (must be >= 3)
    /// * `time_slices` - Number of time slices in the foliation
    /// * `dimension` - Dimension of the triangulation (should be 2)
    ///
    /// # Returns
    ///
    /// An `Option<CausalTriangulation2D>` containing the triangulation if successful.
    /// Returns `None` if triangulation generation fails due to invalid parameters.
    #[must_use]
    pub fn try_new(vertices: u32, time_slices: u32, dimension: u32) -> Option<Self> {
        try_generate_random_delaunay2(vertices).map(|tds| Self {
            tds,
            time_slices,
            dimension,
        })
    }

    /// Creates a new 2D causal triangulation.
    ///
    /// # Panics
    ///
    /// Panics if the underlying triangulation generation fails.
    #[must_use]
    pub fn new(vertices: u32, time_slices: u32, dimension: u32) -> Self {
        let tds = generate_random_delaunay2(vertices);
        Self {
            tds,
            time_slices,
            dimension,
        }
    }
}

/// Generates a random Delaunay triangulation with the specified number of vertices.
///
/// This function creates a proper Delaunay triangulation using the delaunay crate's
/// utility functions.
///
/// # Arguments
///
/// * `number_of_vertices` - The number of vertices to include in the triangulation
///
/// # Returns
///
/// A Tds structure representing the Delaunay triangulation.
///
/// # Panics
///
/// This function panics if the random triangulation generation fails, which can happen
/// if the number of vertices is invalid (< 3) or if the coordinate generation fails.
/// Generates a random Delaunay triangulation with the specified number of vertices.
///
/// This function creates a proper Delaunay triangulation using the delaunay crate's
/// utility functions. Returns `None` if triangulation generation fails.
///
/// # Arguments
///
/// * `number_of_vertices` - The number of vertices to include in the triangulation
///
/// # Returns
///
/// An `Option<Tds>` containing the triangulation if successful, `None` otherwise.
///
/// # Errors
///
/// Returns `None` if the random triangulation generation fails, which can happen
/// if the number of vertices is invalid (< 3) or if coordinate generation fails.
#[must_use]
pub fn try_generate_random_delaunay2(number_of_vertices: u32) -> Option<delaunay::core::Tds<f64, i32, i32, 2>> {
    generate_random_triangulation(number_of_vertices as usize, (0.0, 10.0), None, None).ok()
}

/// Generates a random Delaunay triangulation with the specified number of vertices.
///
/// This function creates a proper Delaunay triangulation using the delaunay crate's
/// utility functions.
///
/// # Arguments
///
/// * `number_of_vertices` - The number of vertices to include in the triangulation
///
/// # Returns
///
/// A Tds structure representing the Delaunay triangulation.
///
/// # Panics
///
/// This function panics if the random triangulation generation fails, which can happen
/// if the number of vertices is invalid (< 3) or if the coordinate generation fails.
#[must_use]
pub fn generate_random_delaunay2(number_of_vertices: u32) -> delaunay::core::Tds<f64, i32, i32, 2> {
    try_generate_random_delaunay2(number_of_vertices)
        .unwrap_or_else(|| panic!("Failed to generate random Delaunay triangulation with {number_of_vertices} vertices"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_causal_triangulation_creation() {
        let triangulation = CausalTriangulation2D::new(10, 3, 2);

        assert_eq!(triangulation.dimension, 2);
        assert_eq!(triangulation.time_slices, 3);
        assert!(triangulation.triangle_count() > 0);
    }

    #[test]
    fn test_vertex_counting() {
        let triangulation = CausalTriangulation2D::new(5, 2, 2);

        // These should work with the actual Tds now
        assert!(triangulation.vertex_count() > 0);
        assert!(triangulation.edge_count() > 0);
        assert!(triangulation.triangle_count() > 0);
    }

    #[test]
    fn test_triangulation_access() {
        let triangulation = CausalTriangulation2D::new(3, 1, 2);

        // Test immutable access
        let tds = triangulation.tds();
        assert!(!tds.cells().is_empty());

        assert!(triangulation.triangle_count() >= 1);
    }

    #[test]
    fn test_from_tds_constructor() {
        let tds = generate_random_delaunay2(5);
        let triangulation = CausalTriangulation2D::from_tds(tds, 2, 2);

        assert_eq!(triangulation.dimension, 2);
        assert_eq!(triangulation.time_slices, 2);
        assert!(triangulation.triangle_count() > 0);
    }

    #[test]
    fn delaunay_triangulation_construction() {
        let triangulation = generate_random_delaunay2(3);

        assert_eq!(triangulation.dim(), 2);
        // For 3 points, we should have 1 triangle
        assert_eq!(triangulation.cells().len(), 1);
    }
}

#[cfg(kani)]
mod verification {
    use super::*;

    #[kani::proof]
    fn triangle_construction() {
        let triangulation = generate_random_delaunay2(3);

        assert!(!triangulation.cells().is_empty());
    }
}
