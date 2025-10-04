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

use crate::geometry::traits::{
    FlipResult, GeometryBackend, SubdivisionResult, TriangulationMut, TriangulationQuery,
};
use std::marker::PhantomData;

/// Delaunay backend wrapping the delaunay crate's Tds
#[derive(Debug)]
pub struct DelaunayBackend<T, VertexData, CellData, const D: usize>
where
    T: delaunay::geometry::CoordinateScalar + 'static,
    VertexData: delaunay::core::DataType + 'static,
    CellData: delaunay::core::DataType + 'static,
    [T; D]: serde::Serialize + for<'de> serde::Deserialize<'de>,
{
    /// The underlying triangulated data structure from delaunay crate
    /// THIS IS THE ONLY PLACE IN THE ENTIRE CDT CODEBASE WHERE `delaunay::Tds` IS USED
    tds: delaunay::core::Tds<T, VertexData, CellData, D>,
    _phantom: PhantomData<(T, VertexData, CellData)>,
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

impl<T, VertexData, CellData, const D: usize> DelaunayBackend<T, VertexData, CellData, D>
where
    T: delaunay::geometry::CoordinateScalar + 'static,
    VertexData: delaunay::core::DataType + 'static,
    CellData: delaunay::core::DataType + 'static,
    [T; D]: serde::Serialize + for<'de> serde::Deserialize<'de>,
{
    /// Create a new Delaunay backend from an existing Tds
    #[must_use]
    pub const fn from_tds(tds: delaunay::core::Tds<T, VertexData, CellData, D>) -> Self {
        Self {
            tds,
            _phantom: PhantomData,
        }
    }

    /// Get a reference to the underlying Tds (for migration purposes only)
    #[must_use]
    pub const fn tds(&self) -> &delaunay::core::Tds<T, VertexData, CellData, D> {
        &self.tds
    }

    /// Get a mutable reference to the underlying Tds (for migration purposes only)
    pub const fn tds_mut(&mut self) -> &mut delaunay::core::Tds<T, VertexData, CellData, D> {
        &mut self.tds
    }
}

impl<T, VertexData, CellData, const D: usize> GeometryBackend
    for DelaunayBackend<T, VertexData, CellData, D>
where
    T: delaunay::geometry::CoordinateScalar + 'static,
    VertexData: delaunay::core::DataType + 'static,
    CellData: delaunay::core::DataType + 'static,
    [T; D]: serde::Serialize + for<'de> serde::Deserialize<'de>,
{
    type Coordinate = T;
    type VertexHandle = DelaunayVertexHandle;
    type EdgeHandle = DelaunayEdgeHandle;
    type FaceHandle = DelaunayFaceHandle;
    type Error = DelaunayError;

    fn backend_name(&self) -> &'static str {
        "delaunay"
    }
}

impl<T, VertexData, CellData, const D: usize> TriangulationQuery
    for DelaunayBackend<T, VertexData, CellData, D>
where
    T: delaunay::geometry::CoordinateScalar + 'static,
    VertexData: delaunay::core::DataType + 'static,
    CellData: delaunay::core::DataType + 'static,
    [T; D]: serde::Serialize + for<'de> serde::Deserialize<'de>,
{
    fn vertex_count(&self) -> usize {
        self.tds.vertices().len()
    }

    fn edge_count(&self) -> usize {
        // Use the canonical edge counting implementation
        crate::triangulations::triangulation::count_edges_in_tds(&self.tds)
    }

    fn face_count(&self) -> usize {
        self.tds.cells().len()
    }

    fn dimension(&self) -> usize {
        D
    }

    fn vertices(&self) -> Box<dyn Iterator<Item = Self::VertexHandle> + '_> {
        // TODO: Implement proper iterator that converts delaunay vertex handles to our opaque handles
        Box::new(std::iter::empty())
    }

    fn edges(&self) -> Box<dyn Iterator<Item = Self::EdgeHandle> + '_> {
        // TODO: Implement proper edge iterator
        Box::new(std::iter::empty())
    }

    fn faces(&self) -> Box<dyn Iterator<Item = Self::FaceHandle> + '_> {
        // TODO: Implement proper face iterator
        Box::new(std::iter::empty())
    }

    fn vertex_coordinates(
        &self,
        _vertex: &Self::VertexHandle,
    ) -> Result<Vec<Self::Coordinate>, Self::Error> {
        // TODO: Implement coordinate lookup
        Err(DelaunayError::OperationFailed(
            "Not implemented".to_string(),
        ))
    }

    fn face_vertices(
        &self,
        _face: &Self::FaceHandle,
    ) -> Result<Vec<Self::VertexHandle>, Self::Error> {
        // TODO: Implement face vertex lookup
        Err(DelaunayError::OperationFailed(
            "Not implemented".to_string(),
        ))
    }

    fn edge_endpoints(
        &self,
        _edge: &Self::EdgeHandle,
    ) -> Result<(Self::VertexHandle, Self::VertexHandle), Self::Error> {
        // TODO: Implement edge endpoint lookup
        Err(DelaunayError::OperationFailed(
            "Not implemented".to_string(),
        ))
    }

    fn adjacent_faces(
        &self,
        _vertex: &Self::VertexHandle,
    ) -> Result<Vec<Self::FaceHandle>, Self::Error> {
        // TODO: Implement adjacency query
        Err(DelaunayError::OperationFailed(
            "Not implemented".to_string(),
        ))
    }

    fn incident_edges(
        &self,
        _vertex: &Self::VertexHandle,
    ) -> Result<Vec<Self::EdgeHandle>, Self::Error> {
        // TODO: Implement incidence query
        Err(DelaunayError::OperationFailed(
            "Not implemented".to_string(),
        ))
    }

    fn face_neighbors(
        &self,
        _face: &Self::FaceHandle,
    ) -> Result<Vec<Self::FaceHandle>, Self::Error> {
        // TODO: Implement neighbor query
        Err(DelaunayError::OperationFailed(
            "Not implemented".to_string(),
        ))
    }

    fn is_valid(&self) -> bool {
        // Basic validation: check that the triangulation has vertices and cells
        !self.tds.vertices().is_empty() && !self.tds.cells().is_empty()
    }
}

impl<T, VertexData, CellData, const D: usize> TriangulationMut
    for DelaunayBackend<T, VertexData, CellData, D>
where
    T: delaunay::geometry::CoordinateScalar + 'static,
    VertexData: delaunay::core::DataType + 'static,
    CellData: delaunay::core::DataType + 'static,
    [T; D]: serde::Serialize + for<'de> serde::Deserialize<'de>,
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
impl<T, VertexData, CellData, const D: usize> DelaunayBackend<T, VertexData, CellData, D>
where
    T: delaunay::geometry::CoordinateScalar
        + std::ops::AddAssign
        + std::ops::SubAssign
        + std::iter::Sum
        + 'static,
    VertexData: delaunay::core::DataType + 'static,
    CellData: delaunay::core::DataType + 'static,
    [T; D]: serde::Serialize + for<'de> serde::Deserialize<'de>,
    for<'a> &'a T: std::ops::Div<T, Output = T>,
{
    /// Check if the triangulation satisfies the Delaunay property using `Tds::is_valid()`
    /// This method is only available for types that satisfy the necessary trait bounds
    #[must_use]
    pub fn is_delaunay(&self) -> bool {
        // Tds::is_valid() returns Result<(), TriangulationValidationError>
        self.tds.is_valid().is_ok()
    }
}

/// Type alias for common 2D Delaunay backend
pub type DelaunayBackend2D = DelaunayBackend<f64, i32, i32, 2>;

// TODO: Add factory functions for creating DelaunayBackend from points
// TODO: Add conversion functions from delaunay vertex/cell handles to opaque handles
// TODO: Implement proper iterator wrappers

#[cfg(test)]
mod tests {
    use super::*;
    use crate::triangulations::triangulation::generate_random_delaunay2;

    #[test]
    fn test_is_delaunay_on_valid_triangulation() {
        // Create a simple valid Delaunay triangulation using the existing utility
        let tds = generate_random_delaunay2(4, (0.0, 10.0));
        let backend = DelaunayBackend::from_tds(tds);

        // Test the specialized is_delaunay method
        assert!(
            backend.is_delaunay(),
            "Valid Delaunay triangulation should pass is_delaunay check"
        );
    }

    #[test]
    fn test_is_delaunay_via_trait() {
        // Create a simple valid Delaunay triangulation
        let tds = generate_random_delaunay2(3, (0.0, 10.0));
        let backend = DelaunayBackend::from_tds(tds);

        // Test is_delaunay through the TriangulationOps trait
        assert!(
            backend.is_delaunay(),
            "Valid triangle should satisfy Delaunay property"
        );
    }

    #[test]
    fn test_is_delaunay_with_multiple_points() {
        // Create a triangulation with more points
        let tds = generate_random_delaunay2(10, (0.0, 100.0));
        let backend = DelaunayBackend::from_tds(tds);

        assert!(
            backend.is_delaunay(),
            "Random point triangulation should be valid Delaunay"
        );
    }

    #[test]
    fn test_is_delaunay_with_many_points() {
        // Create a larger triangulation
        let tds = generate_random_delaunay2(20, (0.0, 50.0));
        let backend = DelaunayBackend::from_tds(tds);

        assert!(
            backend.is_delaunay(),
            "Larger triangulation should be valid Delaunay"
        );
    }

    #[test]
    fn test_is_valid_basic() {
        // Test the basic is_valid implementation
        let tds = generate_random_delaunay2(3, (0.0, 10.0));
        let backend = DelaunayBackend::from_tds(tds);

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
        let tds = generate_random_delaunay2(5, (0.0, 10.0));
        let backend = DelaunayBackend::from_tds(tds);

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
        let tds = generate_random_delaunay2(3, (0.0, 10.0));
        let backend = DelaunayBackend::from_tds(tds);

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
}
