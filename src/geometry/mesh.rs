//! CDT-agnostic mesh data structures.
//!
//! These types provide high-level mesh representations that are independent
//! of the underlying geometry backend.

use super::traits::CoordinateScalar;

/// A vertex in the mesh with its coordinates
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Vertex<T: CoordinateScalar> {
    /// Spatial coordinates of the vertex
    pub coordinates: Vec<T>,
    /// Optional time slice assignment for CDT
    pub time_slice: Option<u32>,
}

/// An edge in the mesh
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Edge {
    /// Indices of the two vertices forming this edge
    pub vertex_indices: (usize, usize),
    /// Whether this edge is timelike (connects different time slices)
    pub is_timelike: bool,
}

/// A face (triangle in 2D, tetrahedron in 3D) in the mesh
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Face {
    /// Indices of vertices forming this face
    pub vertex_indices: Vec<usize>,
}

/// Complete mesh representation
#[derive(Debug, Clone)]
pub struct Mesh<T: CoordinateScalar> {
    /// All vertices in the mesh
    pub vertices: Vec<Vertex<T>>,
    /// All edges in the mesh
    pub edges: Vec<Edge>,
    /// All faces in the mesh
    pub faces: Vec<Face>,
    /// Dimensionality of the mesh
    pub dimension: usize,
}

impl<T: CoordinateScalar> Mesh<T> {
    /// Create a new empty mesh
    #[must_use]
    pub const fn new(dimension: usize) -> Self {
        Self {
            vertices: Vec::new(),
            edges: Vec::new(),
            faces: Vec::new(),
            dimension,
        }
    }

    /// Get the number of vertices
    #[must_use]
    pub const fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    /// Get the number of edges
    #[must_use]
    pub const fn edge_count(&self) -> usize {
        self.edges.len()
    }

    /// Get the number of faces
    #[must_use]
    pub const fn face_count(&self) -> usize {
        self.faces.len()
    }
}
