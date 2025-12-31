//! Delaunay backend - wraps the delaunay crate.
//!
//! This is the ONLY module that directly uses types from the delaunay crate,
//! providing complete isolation of the geometry implementation from CDT logic.
//!
//! # Thread Safety Issue
//!
//! **IMPORTANT**: The underlying `delaunay::core::Tds` does not currently implement
//! `Send` or `Sync` traits, which causes compilation errors when trying to use
//! this backend in multi-threaded contexts.
//!
//! ## TODO: Upstream Fix Required
//!
//! This issue should be resolved by adding `Send + Sync` implementations to the
//! delaunay crate's `Tds` type. Once that's done, this backend will automatically
//! become thread-safe without any changes to this code.
//!
//! **Action Item**: Submit a PR to the delaunay crate to add:
//! ```rust,ignore
//! unsafe impl<T, VD, CD, const D: usize> Send for Tds<T, VD, CD, D>
//! where T: Send, VD: Send, CD: Send { }
//!
//! unsafe impl<T, VD, CD, const D: usize> Sync for Tds<T, VD, CD, D>
//! where T: Sync, VD: Sync, CD: Sync { }
//! ```
//!
//! ## Current Workaround
//!
//! The `GeometryBackend` trait has been designed WITHOUT `Send + Sync` requirements,
//! allowing this backend to compile. An optional `ThreadSafeBackend` marker trait
//! is available for backends that do support threading.
// cspell:ignore vkey

use crate::geometry::traits::{
    FlipResult, GeometryBackend, SubdivisionResult, TriangulationMut, TriangulationQuery,
};
use delaunay::core::delaunay_triangulation::DelaunayTriangulation;
use delaunay::geometry::kernel::FastKernel;
use delaunay::prelude::*;
use std::marker::PhantomData;
use std::sync::Arc;

/// Delaunay backend wrapping the delaunay crate's triangulation (f64 coordinates)
#[derive(Debug)]
pub struct DelaunayBackend<VertexData, CellData, const D: usize>
where
    VertexData: delaunay::core::DataType
        + Copy
        + Clone
        + std::fmt::Debug
        + Eq
        + Ord
        + std::hash::Hash
        + serde::Serialize
        + for<'de> serde::Deserialize<'de>
        + 'static,
    CellData: delaunay::core::DataType
        + Copy
        + Clone
        + std::fmt::Debug
        + Eq
        + Ord
        + std::hash::Hash
        + serde::Serialize
        + for<'de> serde::Deserialize<'de>
        + 'static,
{
    /// The underlying Delaunay triangulation from the delaunay crate
    dt: Arc<DelaunayTriangulation<FastKernel<f64>, VertexData, CellData, D>>,
    _phantom: PhantomData<(VertexData, CellData)>,
}

/// Opaque handle for vertices in Delaunay backend
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DelaunayVertexHandle {
    id: uuid::Uuid,
}

/// Opaque handle for edges in Delaunay backend
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DelaunayEdgeHandle {
    // TODO: Implement proper edge identification
    id: usize,
}

/// Opaque handle for faces in Delaunay backend
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DelaunayFaceHandle {
    id: uuid::Uuid,
}

/// Error type for Delaunay backend operations
#[derive(Debug, thiserror::Error)]
pub enum DelaunayError {
    /// Operation failed with an error message
    #[error("Delaunay operation failed: {0}")]
    OperationFailed(String),

    /// Invalid handle provided
    #[error("Invalid handle: {0}")]
    InvalidHandle(String),

    /// Geometry-related error
    #[error("Geometry error: {0}")]
    GeometryError(String),
}

impl<VertexData, CellData, const D: usize> DelaunayBackend<VertexData, CellData, D>
where
    VertexData: delaunay::core::DataType
        + Copy
        + Clone
        + std::fmt::Debug
        + Eq
        + Ord
        + std::hash::Hash
        + serde::Serialize
        + for<'de> serde::Deserialize<'de>
        + 'static,
    CellData: delaunay::core::DataType
        + Copy
        + Clone
        + std::fmt::Debug
        + Eq
        + Ord
        + std::hash::Hash
        + serde::Serialize
        + for<'de> serde::Deserialize<'de>
        + 'static,
{
    /// Create a new Delaunay backend from an existing Delaunay triangulation
    #[must_use]
    pub fn from_triangulation(
        dt: DelaunayTriangulation<FastKernel<f64>, VertexData, CellData, D>,
    ) -> Self {
        Self {
            dt: Arc::new(dt),
            _phantom: PhantomData,
        }
    }

    /// Access the underlying Delaunay triangulation (read-only)
    #[must_use]
    pub fn triangulation(
        &self,
    ) -> &DelaunayTriangulation<FastKernel<f64>, VertexData, CellData, D> {
        &self.dt
    }

    /// Get a reference to the underlying Tds (for migration purposes only)
    #[must_use]
    pub fn tds(&self) -> &delaunay::core::Tds<f64, VertexData, CellData, D> {
        self.dt.tds()
    }

    #[inline]
    fn vertex_key_from_handle(&self, handle: &DelaunayVertexHandle) -> Option<VertexKey> {
        self.tds().vertex_key_from_uuid(&handle.id)
    }

    #[inline]
    fn cell_key_from_handle(&self, handle: &DelaunayFaceHandle) -> Option<CellKey> {
        self.tds().cell_key_from_uuid(&handle.id)
    }
}

impl<VertexData, CellData, const D: usize> GeometryBackend
    for DelaunayBackend<VertexData, CellData, D>
where
    VertexData: delaunay::core::DataType + 'static,
    CellData: delaunay::core::DataType + 'static,
    [f64; D]: serde::Serialize + for<'de> serde::Deserialize<'de>,
{
    type Coordinate = f64;
    type VertexHandle = DelaunayVertexHandle;
    type EdgeHandle = DelaunayEdgeHandle;
    type FaceHandle = DelaunayFaceHandle;
    type Error = DelaunayError;

    fn backend_name(&self) -> &'static str {
        "delaunay"
    }
}

impl<VertexData, CellData, const D: usize> TriangulationQuery
    for DelaunayBackend<VertexData, CellData, D>
where
    VertexData: delaunay::core::DataType + 'static,
    CellData: delaunay::core::DataType + 'static,
    [f64; D]: serde::Serialize + for<'de> serde::Deserialize<'de>,
{
    fn vertex_count(&self) -> usize {
        self.dt.number_of_vertices()
    }

    /// Count edges in the triangulation.
    ///
    /// **Performance Note**: This is an O(E) operation that scans all facets and builds
    /// a `HashSet` for uniqueness on every call. For performance-critical code performing
    /// frequent edge counts, consider caching the result at the call site or using the
    /// [`refresh_cache()`](crate::cdt::triangulation::CdtTriangulation::refresh_cache)
    /// method available on [`CdtTriangulation`](crate::cdt::triangulation::CdtTriangulation).
    fn edge_count(&self) -> usize {
        // Use canonical edge counting to avoid double-counting shared facets.
        count_edges_in_tds(self.dt.tds())
    }

    fn face_count(&self) -> usize {
        self.dt.number_of_cells()
    }

    fn dimension(&self) -> usize {
        D
    }

    fn vertices(&self) -> Box<dyn Iterator<Item = Self::VertexHandle> + '_> {
        Box::new(
            self.dt
                .tds()
                .vertices()
                .map(|(_, v)| DelaunayVertexHandle { id: v.uuid() }),
        )
    }

    fn edges(&self) -> Box<dyn Iterator<Item = Self::EdgeHandle> + '_> {
        let mut seen = std::collections::HashSet::new();
        let mut handles = Vec::new();

        for (idx, facet_view) in self.dt.facets().enumerate() {
            if let Ok(vertices_iter) = facet_view.vertices() {
                let vertices: Vec<_> = vertices_iter.collect();
                if vertices.len() == 2 {
                    let mut edge = [vertices[0].uuid(), vertices[1].uuid()];
                    edge.sort();
                    if seen.insert(edge) {
                        handles.push(DelaunayEdgeHandle { id: idx });
                    }
                }
            }
        }

        handles.sort_by_key(|h| h.id);
        Box::new(handles.into_iter())
    }

    fn faces(&self) -> Box<dyn Iterator<Item = Self::FaceHandle> + '_> {
        Box::new(
            self.dt
                .tds()
                .cells()
                .map(|(_, c)| DelaunayFaceHandle { id: c.uuid() }),
        )
    }

    fn vertex_coordinates(
        &self,
        vertex: &Self::VertexHandle,
    ) -> Result<Vec<Self::Coordinate>, Self::Error> {
        log::trace!("vertex_coordinates: searching for vertex {}", vertex.id);
        let vkey = self
            .vertex_key_from_handle(vertex)
            .ok_or_else(|| DelaunayError::InvalidHandle("Vertex not found".to_string()))?;
        let v = self
            .dt
            .tds()
            .get_vertex_by_key(vkey)
            .ok_or_else(|| DelaunayError::InvalidHandle("Vertex not found".to_string()))?;

        log::trace!("vertex_coordinates: found vertex {}", vertex.id);

        // Extract coordinates from the point using the Coordinate trait
        let point = v.point();
        let coords_vec: Vec<Self::Coordinate> = point.coords().iter().copied().collect();
        log::debug!(
            "vertex_coordinates: coords() returned {} values {:?}",
            coords_vec.len(),
            &coords_vec
        );
        Ok(coords_vec)
    }

    fn face_vertices(
        &self,
        face: &Self::FaceHandle,
    ) -> Result<Vec<Self::VertexHandle>, Self::Error> {
        let cell_key = self
            .cell_key_from_handle(face)
            .ok_or_else(|| DelaunayError::InvalidHandle("Face not found".to_string()))?;
        let cell = self
            .dt
            .tds()
            .get_cell(cell_key)
            .ok_or_else(|| DelaunayError::InvalidHandle("Face not found".to_string()))?;

        let mut vertices = Vec::new();
        for &vkey in cell.vertices() {
            let v = self
                .dt
                .tds()
                .get_vertex_by_key(vkey)
                .ok_or_else(|| DelaunayError::InvalidHandle("Vertex not found".to_string()))?;
            vertices.push(DelaunayVertexHandle { id: v.uuid() });
        }

        Ok(vertices)
    }

    fn edge_endpoints(
        &self,
        edge: &Self::EdgeHandle,
    ) -> Result<(Self::VertexHandle, Self::VertexHandle), Self::Error> {
        for (idx, facet_view) in self.dt.facets().enumerate() {
            if idx == edge.id
                && let Ok(vertices_iter) = facet_view.vertices()
            {
                let vertices: Vec<_> = vertices_iter.collect();
                if vertices.len() == 2 {
                    return Ok((
                        DelaunayVertexHandle {
                            id: vertices[0].uuid(),
                        },
                        DelaunayVertexHandle {
                            id: vertices[1].uuid(),
                        },
                    ));
                }
            }
        }

        Err(DelaunayError::InvalidHandle("Edge not found".to_string()))
    }

    fn adjacent_faces(
        &self,
        vertex: &Self::VertexHandle,
    ) -> Result<Vec<Self::FaceHandle>, Self::Error> {
        let vkey = self
            .vertex_key_from_handle(vertex)
            .ok_or_else(|| DelaunayError::InvalidHandle("Vertex not found".to_string()))?;

        let mut adjacent = Vec::new();
        for (_, cell) in self.dt.tds().cells() {
            if cell.contains_vertex(vkey) {
                adjacent.push(DelaunayFaceHandle { id: cell.uuid() });
            }
        }

        Ok(adjacent)
    }

    fn incident_edges(
        &self,
        vertex: &Self::VertexHandle,
    ) -> Result<Vec<Self::EdgeHandle>, Self::Error> {
        // Find all edges that contain this vertex
        let mut unique_edges = std::collections::HashMap::new();
        let all_facets = self.dt.facets();

        for (idx, facet_view) in all_facets.enumerate() {
            if let Ok(vertices_iter) = facet_view.vertices() {
                let vertices: Vec<_> = vertices_iter.collect();
                if vertices.len() == 2 {
                    let uuid1 = vertices[0].uuid();
                    let uuid2 = vertices[1].uuid();

                    if uuid1 == vertex.id || uuid2 == vertex.id {
                        // Create a sorted edge key to ensure uniqueness
                        let mut edge = [uuid1, uuid2];
                        edge.sort();
                        unique_edges.entry(edge).or_insert(idx);
                    }
                }
            }
        }

        Ok(unique_edges
            .into_values()
            .map(|id| DelaunayEdgeHandle { id })
            .collect())
    }

    fn face_neighbors(
        &self,
        face: &Self::FaceHandle,
    ) -> Result<Vec<Self::FaceHandle>, Self::Error> {
        let cell_key = self
            .cell_key_from_handle(face)
            .ok_or_else(|| DelaunayError::InvalidHandle("Face not found".to_string()))?;
        let cell = self
            .tds()
            .get_cell(cell_key)
            .ok_or_else(|| DelaunayError::InvalidHandle("Face not found".to_string()))?;

        // Get neighbors from the cell's neighbors() method
        // Phase 3A: neighbors() now returns CellKeys, need to look up UUIDs via TDS
        let mut neighbors = Vec::new();
        if let Some(neighbor_keys) = cell.neighbors() {
            for neighbor_key in neighbor_keys.iter().copied().flatten() {
                if let Some(neighbor_cell) = self.dt.tds().get_cell(neighbor_key) {
                    neighbors.push(DelaunayFaceHandle {
                        id: neighbor_cell.uuid(),
                    });
                }
            }
        }

        Ok(neighbors)
    }

    fn is_valid(&self) -> bool {
        self.dt.is_valid().is_ok()
    }
}

impl<VertexData, CellData, const D: usize> TriangulationMut
    for DelaunayBackend<VertexData, CellData, D>
where
    VertexData: delaunay::core::DataType + 'static,
    CellData: delaunay::core::DataType + 'static,
    [f64; D]: serde::Serialize + for<'de> serde::Deserialize<'de>,
{
    fn insert_vertex(
        &mut self,
        _coords: &[Self::Coordinate],
    ) -> Result<Self::VertexHandle, Self::Error> {
        // TODO: Implement vertex insertion
        Err(DelaunayError::OperationFailed(
            "Not implemented".to_string(),
        ))
    }

    fn remove_vertex(
        &mut self,
        _vertex: Self::VertexHandle,
    ) -> Result<Vec<Self::FaceHandle>, Self::Error> {
        // TODO: Implement vertex removal
        Err(DelaunayError::OperationFailed(
            "Not implemented".to_string(),
        ))
    }

    fn move_vertex(
        &mut self,
        _vertex: Self::VertexHandle,
        _new_coords: &[Self::Coordinate],
    ) -> Result<(), Self::Error> {
        // TODO: Implement vertex movement
        Err(DelaunayError::OperationFailed(
            "Not implemented".to_string(),
        ))
    }

    fn flip_edge(
        &mut self,
        _edge: Self::EdgeHandle,
    ) -> Result<FlipResult<Self::VertexHandle, Self::EdgeHandle, Self::FaceHandle>, Self::Error>
    {
        // TODO: Implement edge flip
        Err(DelaunayError::OperationFailed(
            "Not implemented".to_string(),
        ))
    }

    fn can_flip_edge(&self, _edge: &Self::EdgeHandle) -> bool {
        // TODO: Implement flip check
        false
    }

    fn subdivide_face(
        &mut self,
        _face: Self::FaceHandle,
        _point: &[Self::Coordinate],
    ) -> Result<
        SubdivisionResult<Self::VertexHandle, Self::EdgeHandle, Self::FaceHandle>,
        Self::Error,
    > {
        // TODO: Implement face subdivision
        Err(DelaunayError::OperationFailed(
            "Not implemented".to_string(),
        ))
    }

    fn clear(&mut self) {
        // TODO: Implement clear operation
    }

    fn reserve_capacity(&mut self, _vertices: usize, _faces: usize) {
        // TODO: Implement capacity reservation
    }
}

// Additional implementation for types that support full Delaunay validation
impl<VertexData, CellData, const D: usize> DelaunayBackend<VertexData, CellData, D>
where
    VertexData: delaunay::core::DataType
        + Copy
        + Clone
        + std::fmt::Debug
        + Eq
        + Ord
        + std::hash::Hash
        + serde::Serialize
        + for<'de> serde::Deserialize<'de>
        + 'static,
    CellData: delaunay::core::DataType
        + Copy
        + Clone
        + std::fmt::Debug
        + Eq
        + Ord
        + std::hash::Hash
        + serde::Serialize
        + for<'de> serde::Deserialize<'de>
        + 'static,
    [f64; D]: serde::Serialize + for<'de> serde::Deserialize<'de>,
{
    /// Check if the triangulation satisfies the Delaunay property using `Tds::is_valid()`
    /// This method is only available for types that satisfy the necessary trait bounds
    #[must_use]
    pub fn is_delaunay(&self) -> bool {
        // Tds::is_valid() returns Result<(), TriangulationValidationError>
        self.dt.is_valid().is_ok()
    }
}

/// Type alias for common 2D Delaunay backend
pub type DelaunayBackend2D = DelaunayBackend<i32, i32, 2>;

/// Counts edges in any Tds structure using a consistent algorithm.
/// This is the canonical edge counting implementation used throughout the codebase.
#[must_use]
pub fn count_edges_in_tds<T, VertexData, CellData, const D: usize>(
    tds: &delaunay::core::Tds<T, VertexData, CellData, D>,
) -> usize
where
    T: delaunay::geometry::CoordinateScalar,
    VertexData: delaunay::core::DataType,
    CellData: delaunay::core::DataType,
    [T; D]: serde::Serialize + for<'de> serde::Deserialize<'de>,
{
    if tds.number_of_vertices() < 2 || tds.number_of_cells() == 0 {
        return 0;
    }

    // Use AllFacetsIter to iterate over all facets (edges in 2D)
    // and count unique edges by tracking vertex pairs
    let mut unique_edges = std::collections::HashSet::new();
    let all_facets = delaunay::core::facet::AllFacetsIter::new(tds);

    for facet_view in all_facets {
        // Get the vertices of this facet (edge in 2D)
        if let Ok(vertices_iter) = facet_view.vertices() {
            let vertices: Vec<_> = vertices_iter.collect();
            if vertices.len() == 2 {
                // Use UUID for unique vertex identification
                let uuid1 = vertices[0].uuid();
                let uuid2 = vertices[1].uuid();
                let mut edge = [uuid1, uuid2];
                edge.sort();
                unique_edges.insert(edge);
            }
        }
    }

    unique_edges.len()
}

// TODO: Add factory functions for creating DelaunayBackend from points
// TODO: Add conversion functions from delaunay vertex/cell handles to opaque handles
// TODO: Implement proper iterator wrappers

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_delaunay_on_valid_triangulation() {
        // Create a simple valid Delaunay triangulation using the existing utility
        let dt = crate::util::generate_random_delaunay2(4, (0.0, 10.0));
        let backend = DelaunayBackend::from_triangulation(dt);

        // Test the specialized is_delaunay method
        assert!(
            backend.is_delaunay(),
            "Valid Delaunay triangulation should pass is_delaunay check"
        );
    }

    #[test]
    fn test_is_delaunay_via_trait() {
        // Create a simple valid Delaunay triangulation
        let dt = crate::util::generate_random_delaunay2(3, (0.0, 10.0));
        let backend = DelaunayBackend::from_triangulation(dt);

        // Test is_delaunay through the TriangulationOps trait
        assert!(
            backend.is_delaunay(),
            "Valid triangle should satisfy Delaunay property"
        );
    }

    #[test]
    fn test_is_delaunay_with_multiple_points() {
        // Create a triangulation with more points
        let dt = crate::util::generate_random_delaunay2(10, (0.0, 100.0));
        let backend = DelaunayBackend::from_triangulation(dt);

        assert!(
            backend.is_delaunay(),
            "Random point triangulation should be valid Delaunay"
        );
    }

    #[test]
    fn test_is_delaunay_with_many_points() {
        // Create a larger triangulation
        let dt = crate::util::generate_random_delaunay2(20, (0.0, 50.0));
        let backend = DelaunayBackend::from_triangulation(dt);

        assert!(
            backend.is_delaunay(),
            "Larger triangulation should be valid Delaunay"
        );
    }

    #[test]
    fn test_is_valid_basic() {
        // Test the basic is_valid implementation
        let dt = crate::util::generate_random_delaunay2(3, (0.0, 10.0));
        let backend = DelaunayBackend::from_triangulation(dt);

        assert!(
            backend.is_valid(),
            "Triangulation with vertices and cells should be valid"
        );
        assert_eq!(backend.vertex_count(), 3, "Should have 3 vertices");
        assert!(backend.face_count() > 0, "Should have at least one face");
    }

    #[test]
    fn test_is_delaunay_consistency() {
        // Test that is_delaunay and is_valid are consistent for valid triangulations
        let dt = crate::util::generate_random_delaunay2(5, (0.0, 10.0));
        let backend = DelaunayBackend::from_triangulation(dt);

        let is_valid = backend.is_valid();
        let is_delaunay = backend.is_delaunay();

        assert!(is_valid, "Triangulation should be valid");
        assert!(
            is_delaunay,
            "Valid Delaunay triangulation should pass is_delaunay"
        );
    }

    #[test]
    fn test_is_delaunay_minimal_triangulation() {
        // Test with minimal triangulation (3 vertices)
        let dt = crate::util::generate_random_delaunay2(3, (0.0, 10.0));
        let backend = DelaunayBackend::from_triangulation(dt);

        assert!(backend.is_valid(), "Minimal triangulation should be valid");
        assert!(
            backend.is_delaunay(),
            "Minimal triangulation should satisfy Delaunay property"
        );
        assert_eq!(backend.vertex_count(), 3, "Should have exactly 3 vertices");
        assert_eq!(
            backend.face_count(),
            1,
            "Should have exactly 1 face (triangle)"
        );
    }

    // Tests for iterator methods

    #[test]
    fn test_vertices_iterator() {
        let dt = crate::util::generate_random_delaunay2(5, (0.0, 10.0));
        let backend = DelaunayBackend::from_triangulation(dt);

        let vertices: Vec<_> = backend.vertices().collect();
        assert_eq!(
            vertices.len(),
            backend.vertex_count(),
            "Iterator should return all vertices"
        );

        // Check that all handles are unique
        let unique_count = vertices
            .iter()
            .collect::<std::collections::HashSet<_>>()
            .len();
        assert_eq!(
            unique_count,
            vertices.len(),
            "All vertex handles should be unique"
        );
    }

    #[test]
    fn test_edges_iterator() {
        let dt = crate::util::generate_random_delaunay2(4, (0.0, 10.0));
        let backend = DelaunayBackend::from_triangulation(dt);

        let edges: Vec<_> = backend.edges().collect();
        assert_eq!(
            edges.len(),
            backend.edge_count(),
            "Iterator should return all edges"
        );

        // Check that all handles are unique
        let unique_count = edges.iter().collect::<std::collections::HashSet<_>>().len();
        assert_eq!(
            unique_count,
            edges.len(),
            "All edge handles should be unique"
        );
    }

    #[test]
    fn test_faces_iterator() {
        let dt = crate::util::generate_random_delaunay2(5, (0.0, 10.0));
        let backend = DelaunayBackend::from_triangulation(dt);

        let faces: Vec<_> = backend.faces().collect();
        assert_eq!(
            faces.len(),
            backend.face_count(),
            "Iterator should return all faces"
        );

        // Check that all handles are unique
        let unique_count = faces.iter().collect::<std::collections::HashSet<_>>().len();
        assert_eq!(
            unique_count,
            faces.len(),
            "All face handles should be unique"
        );
    }

    // Tests for query methods

    #[test]
    fn test_vertex_coordinates() {
        let dt = crate::util::generate_random_delaunay2(3, (0.0, 10.0));
        let backend = DelaunayBackend::from_triangulation(dt);

        let vertices: Vec<_> = backend.vertices().collect();
        assert!(!vertices.is_empty(), "Should have at least one vertex");

        for vertex in &vertices {
            let coords = backend
                .vertex_coordinates(vertex)
                .expect("Should retrieve coordinates for valid vertex");
            assert_eq!(coords.len(), 2, "Should have 2D coordinates");
            assert!(
                coords.iter().all(|&c| (0.0..=10.0).contains(&c)),
                "Coordinates should be within expected range"
            );
        }
    }

    #[test]
    fn test_vertex_coordinates_invalid_handle() {
        let dt = crate::util::generate_random_delaunay2(3, (0.0, 10.0));
        let backend = DelaunayBackend::from_triangulation(dt);

        let invalid_handle = DelaunayVertexHandle {
            id: uuid::Uuid::new_v4(),
        };
        let result = backend.vertex_coordinates(&invalid_handle);
        assert!(result.is_err(), "Should error for invalid vertex handle");
    }

    #[test]
    fn test_face_vertices() {
        let dt = crate::util::generate_random_delaunay2(3, (0.0, 10.0));
        let backend = DelaunayBackend::from_triangulation(dt);

        let faces: Vec<_> = backend.faces().collect();
        assert!(!faces.is_empty(), "Should have at least one face");

        for face in &faces {
            let vertices = backend
                .face_vertices(face)
                .expect("Should retrieve vertices for valid face");
            assert_eq!(vertices.len(), 3, "2D face should have exactly 3 vertices");

            // Verify all vertices are unique
            let unique_count = vertices
                .iter()
                .collect::<std::collections::HashSet<_>>()
                .len();
            assert_eq!(
                unique_count,
                vertices.len(),
                "Face vertices should be unique"
            );
        }
    }

    #[test]
    fn test_face_vertices_invalid_handle() {
        let dt = crate::util::generate_random_delaunay2(3, (0.0, 10.0));
        let backend = DelaunayBackend::from_triangulation(dt);

        let invalid_handle = DelaunayFaceHandle {
            id: uuid::Uuid::new_v4(),
        };
        let result = backend.face_vertices(&invalid_handle);
        assert!(result.is_err(), "Should error for invalid face handle");
    }

    #[test]
    fn test_edge_endpoints() {
        let dt = crate::util::generate_random_delaunay2(4, (0.0, 10.0));
        let backend = DelaunayBackend::from_triangulation(dt);

        let edges: Vec<_> = backend.edges().collect();
        assert!(!edges.is_empty(), "Should have at least one edge");

        for edge in &edges {
            let (v1, v2) = backend
                .edge_endpoints(edge)
                .expect("Should retrieve endpoints for valid edge");
            assert_ne!(v1, v2, "Edge endpoints should be different");

            // Verify endpoints exist in vertex list
            let vertices: Vec<_> = backend.vertices().collect();
            assert!(
                vertices.contains(&v1),
                "First endpoint should be a valid vertex"
            );
            assert!(
                vertices.contains(&v2),
                "Second endpoint should be a valid vertex"
            );
        }
    }

    #[test]
    fn test_edge_endpoints_invalid_handle() {
        let dt = crate::util::generate_random_delaunay2(3, (0.0, 10.0));
        let backend = DelaunayBackend::from_triangulation(dt);

        let invalid_handle = DelaunayEdgeHandle { id: 999_999 };
        let result = backend.edge_endpoints(&invalid_handle);
        assert!(result.is_err(), "Should error for invalid edge handle");
    }

    #[test]
    fn test_adjacent_faces() {
        let dt = crate::util::generate_random_delaunay2(4, (0.0, 10.0));
        let backend = DelaunayBackend::from_triangulation(dt);

        let vertices: Vec<_> = backend.vertices().collect();
        assert!(!vertices.is_empty(), "Should have at least one vertex");

        for vertex in &vertices {
            let adjacent = backend
                .adjacent_faces(vertex)
                .expect("Should retrieve adjacent faces for valid vertex");
            assert!(
                !adjacent.is_empty(),
                "Each vertex should have at least one adjacent face"
            );

            // Verify each adjacent face contains this vertex
            for face_handle in &adjacent {
                let face_vertices = backend
                    .face_vertices(face_handle)
                    .expect("Should retrieve face vertices");
                assert!(
                    face_vertices.contains(vertex),
                    "Adjacent face should contain the vertex"
                );
            }
        }
    }

    #[test]
    fn test_incident_edges() {
        let dt = crate::util::generate_random_delaunay2(4, (0.0, 10.0));
        let backend = DelaunayBackend::from_triangulation(dt);

        let vertices: Vec<_> = backend.vertices().collect();
        assert!(!vertices.is_empty(), "Should have at least one vertex");

        for vertex in &vertices {
            let incident = backend
                .incident_edges(vertex)
                .expect("Should retrieve incident edges for valid vertex");
            assert!(
                !incident.is_empty(),
                "Each vertex should have at least one incident edge"
            );

            // Verify each incident edge has this vertex as an endpoint
            for edge_handle in &incident {
                let (v1, v2) = backend
                    .edge_endpoints(edge_handle)
                    .expect("Should retrieve edge endpoints");
                assert!(
                    v1 == *vertex || v2 == *vertex,
                    "Incident edge should have vertex as an endpoint"
                );
            }
        }
    }

    #[test]
    fn test_face_neighbors() {
        let dt = crate::util::generate_random_delaunay2(5, (0.0, 10.0));
        let backend = DelaunayBackend::from_triangulation(dt);

        let faces: Vec<_> = backend.faces().collect();
        assert!(!faces.is_empty(), "Should have at least one face");

        for face in &faces {
            let neighbors = backend
                .face_neighbors(face)
                .expect("Should retrieve neighbors for valid face");

            // In a 2D triangulation, each face can have 0-3 neighbors
            assert!(
                neighbors.len() <= 3,
                "A 2D face should have at most 3 neighbors"
            );

            // Verify neighbors are valid faces
            let all_faces: std::collections::HashSet<_> = backend.faces().collect();
            for neighbor in &neighbors {
                assert!(
                    all_faces.contains(neighbor),
                    "Neighbor should be a valid face"
                );
            }
        }
    }

    #[test]
    fn test_face_neighbors_invalid_handle() {
        let dt = crate::util::generate_random_delaunay2(3, (0.0, 10.0));
        let backend = DelaunayBackend::from_triangulation(dt);

        let invalid_handle = DelaunayFaceHandle {
            id: uuid::Uuid::new_v4(),
        };
        let result = backend.face_neighbors(&invalid_handle);
        assert!(result.is_err(), "Should error for invalid face handle");
    }

    #[test]
    fn test_topology_consistency() {
        // Test that topology is consistent across different query methods
        // Use a fixed seed for reproducibility and to avoid random topology issues
        let dt = delaunay::geometry::util::generate_random_triangulation::<f64, i32, i32, 2>(
            6,
            (0.0, 10.0),
            None,
            Some(42),
        )
        .expect("Failed to generate triangulation with fixed seed");
        let backend = DelaunayBackend::from_triangulation(dt);

        let vertex_count = backend.vertex_count();
        let edge_count = backend.edge_count();
        let face_count = backend.face_count();

        // Verify Euler characteristic for planar graphs
        // For a triangulation without the outer infinite face: V - E + F = 1
        // For a triangulation with the outer infinite face: V - E + F = 2
        // Note: Random triangulations may occasionally have degeneracies that result in χ = 0
        #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
        let euler = vertex_count as i32 - edge_count as i32 + face_count as i32;
        assert!(
            (0..=2).contains(&euler),
            "Euler characteristic should be in range [0, 2] for planar triangulation, got {euler} (V={vertex_count}, E={edge_count}, F={face_count})"
        );

        // Count edges through incident_edges (should match total edge count)
        let mut edge_set = std::collections::HashSet::new();
        for vertex in backend.vertices() {
            if let Ok(incident) = backend.incident_edges(&vertex) {
                edge_set.extend(incident);
            }
        }
        assert_eq!(
            edge_set.len(),
            edge_count,
            "Total edges from incident_edges should match edge_count"
        );
    }

    #[test]
    fn test_minimal_triangulation_queries() {
        // Test with minimal valid triangulation (3 vertices, 1 face)
        let dt = crate::util::generate_random_delaunay2(3, (0.0, 10.0));
        let backend = DelaunayBackend::from_triangulation(dt);

        // Test all vertices are accessible
        let vertices: Vec<_> = backend.vertices().collect();
        assert_eq!(vertices.len(), 3, "Should have exactly 3 vertices");

        // Test all edges are accessible
        let edges: Vec<_> = backend.edges().collect();
        assert_eq!(edges.len(), 3, "Should have exactly 3 edges");

        // Test face is accessible
        let faces: Vec<_> = backend.faces().collect();
        assert_eq!(faces.len(), 1, "Should have exactly 1 face");

        // Verify face has all 3 vertices
        let face_vertices = backend
            .face_vertices(&faces[0])
            .expect("Should get face vertices");
        assert_eq!(face_vertices.len(), 3, "Face should have 3 vertices");
    }
}
