//! Main triangulation module for CDT.
//!
//! This module provides the core triangulation functionality for
//! Causal Dynamical Triangulations, including integration with
//! the delaunay crate and CDT-specific data structures.

use crate::triangulations::delaunay_triangulations::generate_random_delaunay2;

/// Main triangulation structure for CDT simulations.
#[derive(Debug, Clone)]
pub struct CausalTriangulation {
    /// The underlying triangulation as vertex indices
    pub triangulation: Vec<Vec<usize>>,
    /// Number of time slices in the foliation
    pub time_slices: u32,
    /// Dimension of the triangulation (2 for current implementation)
    pub dimension: u32,
}

impl CausalTriangulation {
    /// Creates a new causal triangulation.
    #[must_use]
    pub fn new(vertices: u32, time_slices: u32, dimension: u32) -> Self {
        let triangulation = generate_random_delaunay2(vertices);

        Self {
            triangulation,
            time_slices,
            dimension,
        }
    }

    /// Returns the number of triangles in the triangulation.
    #[must_use]
    pub const fn triangle_count(&self) -> usize {
        self.triangulation.len()
    }

    /// Returns the number of unique vertices in the triangulation.
    #[must_use]
    pub fn vertex_count(&self) -> usize {
        let mut vertices = std::collections::HashSet::new();
        for triangle in &self.triangulation {
            for &vertex in triangle {
                vertices.insert(vertex);
            }
        }
        vertices.len()
    }

    /// Returns the number of edges in the triangulation.
    #[must_use]
    pub fn edge_count(&self) -> usize {
        let mut edges = std::collections::HashSet::new();
        for triangle in &self.triangulation {
            if triangle.len() >= 3 {
                for i in 0..3 {
                    let v1 = triangle[i];
                    let v2 = triangle[(i + 1) % 3];
                    // Store edges in canonical form (smaller index first)
                    let edge = if v1 < v2 { (v1, v2) } else { (v2, v1) };
                    edges.insert(edge);
                }
            }
        }
        edges.len()
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

    /// Returns a reference to the underlying triangulation.
    #[must_use]
    pub const fn triangles(&self) -> &Vec<Vec<usize>> {
        &self.triangulation
    }

    /// Returns a mutable reference to the underlying triangulation.
    pub const fn triangles_mut(&mut self) -> &mut Vec<Vec<usize>> {
        &mut self.triangulation
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_causal_triangulation_creation() {
        let triangulation = CausalTriangulation::new(10, 3, 2);

        assert_eq!(triangulation.dimension, 2);
        assert_eq!(triangulation.time_slices, 3);
        assert!(!triangulation.triangulation.is_empty());
    }

    #[test]
    fn test_vertex_counting() {
        let mut triangulation = CausalTriangulation::new(5, 2, 2);

        // Override with known triangulation for testing
        triangulation.triangulation = vec![vec![0, 1, 2], vec![1, 2, 3]];

        assert_eq!(triangulation.vertex_count(), 4);
        assert_eq!(triangulation.edge_count(), 5);
        assert_eq!(triangulation.triangle_count(), 2);
    }

    #[test]
    fn test_triangulation_access() {
        let mut triangulation = CausalTriangulation::new(3, 1, 2);

        // Test immutable access
        let triangles = triangulation.triangles();
        assert!(!triangles.is_empty());

        // Test mutable access
        let triangles_mut = triangulation.triangles_mut();
        triangles_mut.push(vec![0, 1, 2]);

        assert!(triangulation.triangle_count() >= 1);
    }
}
