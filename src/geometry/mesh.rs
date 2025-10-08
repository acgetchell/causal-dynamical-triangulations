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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vertex_creation() {
        let vertex = Vertex {
            coordinates: vec![1.0, 2.0],
            time_slice: Some(0),
        };

        assert_eq!(vertex.coordinates, vec![1.0, 2.0]);
        assert_eq!(vertex.time_slice, Some(0));
    }

    #[test]
    fn test_vertex_without_time_slice() {
        let vertex = Vertex {
            coordinates: vec![0.0, 0.0, 1.0],
            time_slice: None,
        };

        assert_eq!(vertex.coordinates.len(), 3);
        assert_eq!(vertex.time_slice, None);
    }

    #[test]
    fn test_edge_creation() {
        let edge = Edge {
            vertex_indices: (0, 1),
            is_timelike: false,
        };

        assert_eq!(edge.vertex_indices, (0, 1));
        assert!(!edge.is_timelike);
    }

    #[test]
    fn test_timelike_edge() {
        let edge = Edge {
            vertex_indices: (2, 5),
            is_timelike: true,
        };

        assert_eq!(edge.vertex_indices, (2, 5));
        assert!(edge.is_timelike);
    }

    #[test]
    fn test_face_creation() {
        let face = Face {
            vertex_indices: vec![0, 1, 2],
        };

        assert_eq!(face.vertex_indices, vec![0, 1, 2]);
        assert_eq!(face.vertex_indices.len(), 3);
    }

    #[test]
    fn test_face_triangle() {
        let triangle = Face {
            vertex_indices: vec![0, 1, 2],
        };

        assert_eq!(triangle.vertex_indices.len(), 3);
    }

    #[test]
    fn test_face_tetrahedron() {
        let tetrahedron = Face {
            vertex_indices: vec![0, 1, 2, 3],
        };

        assert_eq!(tetrahedron.vertex_indices.len(), 4);
    }

    #[test]
    fn test_empty_mesh_creation() {
        let mesh: Mesh<f64> = Mesh::new(2);

        assert_eq!(mesh.dimension, 2);
        assert_eq!(mesh.vertex_count(), 0);
        assert_eq!(mesh.edge_count(), 0);
        assert_eq!(mesh.face_count(), 0);
        assert!(mesh.vertices.is_empty());
        assert!(mesh.edges.is_empty());
        assert!(mesh.faces.is_empty());
    }

    #[test]
    fn test_mesh_3d() {
        let mesh: Mesh<f32> = Mesh::new(3);
        assert_eq!(mesh.dimension, 3);
    }

    #[test]
    fn test_mesh_with_vertices() {
        let mut mesh: Mesh<f64> = Mesh::new(2);

        mesh.vertices.push(Vertex {
            coordinates: vec![0.0, 0.0],
            time_slice: Some(0),
        });
        mesh.vertices.push(Vertex {
            coordinates: vec![1.0, 0.0],
            time_slice: Some(0),
        });
        mesh.vertices.push(Vertex {
            coordinates: vec![0.5, 1.0],
            time_slice: Some(1),
        });

        assert_eq!(mesh.vertex_count(), 3);
        assert_eq!(mesh.vertices[0].coordinates, vec![0.0, 0.0]);
        assert_eq!(mesh.vertices[2].time_slice, Some(1));
    }

    #[test]
    fn test_mesh_with_edges() {
        let mut mesh: Mesh<f64> = Mesh::new(2);

        mesh.edges.push(Edge {
            vertex_indices: (0, 1),
            is_timelike: false,
        });
        mesh.edges.push(Edge {
            vertex_indices: (1, 2),
            is_timelike: true,
        });

        assert_eq!(mesh.edge_count(), 2);
        assert!(!mesh.edges[0].is_timelike);
        assert!(mesh.edges[1].is_timelike);
    }

    #[test]
    fn test_mesh_with_faces() {
        let mut mesh: Mesh<f64> = Mesh::new(2);

        mesh.faces.push(Face {
            vertex_indices: vec![0, 1, 2],
        });
        mesh.faces.push(Face {
            vertex_indices: vec![1, 2, 3],
        });

        assert_eq!(mesh.face_count(), 2);
        assert_eq!(mesh.faces[0].vertex_indices, vec![0, 1, 2]);
        assert_eq!(mesh.faces[1].vertex_indices, vec![1, 2, 3]);
    }

    #[test]
    fn test_complete_mesh() {
        let mut mesh: Mesh<f64> = Mesh::new(2);

        // Add vertices
        mesh.vertices.push(Vertex {
            coordinates: vec![0.0, 0.0],
            time_slice: Some(0),
        });
        mesh.vertices.push(Vertex {
            coordinates: vec![1.0, 0.0],
            time_slice: Some(0),
        });
        mesh.vertices.push(Vertex {
            coordinates: vec![0.5, 1.0],
            time_slice: Some(1),
        });

        // Add edges
        mesh.edges.push(Edge {
            vertex_indices: (0, 1),
            is_timelike: false,
        });
        mesh.edges.push(Edge {
            vertex_indices: (0, 2),
            is_timelike: true,
        });
        mesh.edges.push(Edge {
            vertex_indices: (1, 2),
            is_timelike: true,
        });

        // Add face
        mesh.faces.push(Face {
            vertex_indices: vec![0, 1, 2],
        });

        assert_eq!(mesh.vertex_count(), 3);
        assert_eq!(mesh.edge_count(), 3);
        assert_eq!(mesh.face_count(), 1);
        assert_eq!(mesh.dimension, 2);

        // Verify connectivity
        let face = &mesh.faces[0];
        assert_eq!(face.vertex_indices.len(), 3);
        assert!(face.vertex_indices.contains(&0));
        assert!(face.vertex_indices.contains(&1));
        assert!(face.vertex_indices.contains(&2));
    }

    #[test]
    fn test_vertex_equality() {
        let v1 = Vertex {
            coordinates: vec![1.0, 2.0],
            time_slice: Some(0),
        };
        let v2 = Vertex {
            coordinates: vec![1.0, 2.0],
            time_slice: Some(0),
        };
        let v3 = Vertex {
            coordinates: vec![1.0, 2.0],
            time_slice: Some(1),
        };

        assert_eq!(v1, v2);
        assert_ne!(v1, v3);
    }

    #[test]
    fn test_edge_equality() {
        let e1 = Edge {
            vertex_indices: (0, 1),
            is_timelike: false,
        };
        let e2 = Edge {
            vertex_indices: (0, 1),
            is_timelike: false,
        };
        let e3 = Edge {
            vertex_indices: (0, 1),
            is_timelike: true,
        };

        assert_eq!(e1, e2);
        assert_ne!(e1, e3);
    }

    #[test]
    fn test_face_equality() {
        let f1 = Face {
            vertex_indices: vec![0, 1, 2],
        };
        let f2 = Face {
            vertex_indices: vec![0, 1, 2],
        };
        let f3 = Face {
            vertex_indices: vec![0, 2, 1],
        };

        assert_eq!(f1, f2);
        assert_ne!(f1, f3);
    }
}
