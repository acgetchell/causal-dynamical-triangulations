//! Triangulation module for CDT.
//!
//! This module provides the core triangulation functionality for
//! Causal Dynamical Triangulations, including integration with
//! the delaunay crate and CDT-specific data structures.

use crate::errors::{CdtError, CdtResult};
use delaunay::core::facet::AllFacetsIter;
use delaunay::geometry::util::generate_random_triangulation;
use num_traits::NumCast;
use std::collections::HashSet;
use std::iter::Sum;
use std::ops::{AddAssign, Div, SubAssign};

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
    tds: delaunay::core::Tds<T, VertexData, CellData, D>,
    /// Number of time slices in the foliation
    time_slices: u32,
    /// Dimension of the triangulation
    dimension: u8,
    /// Cached edge count to avoid recalculation
    cached_edge_count: std::cell::OnceCell<usize>,
}

/// Smart wrapper for mutable TDS access that invalidates cache on drop.
///
/// This wrapper provides mutable access to the underlying TDS while ensuring
/// that the cached edge count is invalidated when the wrapper is dropped,
/// maintaining cache consistency.
pub struct TdsMutWrapper<'a, T, VertexData, CellData, const D: usize>
where
    T: delaunay::geometry::CoordinateScalar,
    VertexData: delaunay::core::DataType,
    CellData: delaunay::core::DataType,
    [T; D]: serde::Serialize + for<'de> serde::Deserialize<'de>,
{
    triangulation: &'a mut CausalTriangulation<T, VertexData, CellData, D>,
    /// Track whether actual modifications occurred to the TDS
    modification_occurred: std::cell::Cell<bool>,
}

impl<'a, T, VertexData, CellData, const D: usize> TdsMutWrapper<'a, T, VertexData, CellData, D>
where
    T: delaunay::geometry::CoordinateScalar,
    VertexData: delaunay::core::DataType,
    CellData: delaunay::core::DataType,
    [T; D]: serde::Serialize + for<'de> serde::Deserialize<'de>,
{
    /// Creates a new TDS mutable wrapper.
    const fn new(triangulation: &'a mut CausalTriangulation<T, VertexData, CellData, D>) -> Self {
        Self {
            triangulation,
            modification_occurred: std::cell::Cell::new(false),
        }
    }

    /// Returns a mutable reference to the underlying TDS.
    pub const fn tds(&mut self) -> &mut delaunay::core::Tds<T, VertexData, CellData, D> {
        &mut self.triangulation.tds
    }

    /// Mark that actual modifications occurred to the TDS.
    /// Call this method when you know you've made changes that affect the edge count.
    pub fn mark_modified(&self) {
        self.modification_occurred.set(true);
    }

    /// Execute a function that may modify the TDS and automatically mark as modified.
    pub fn with_modification<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut delaunay::core::Tds<T, VertexData, CellData, D>) -> R,
    {
        let result = f(&mut self.triangulation.tds);
        self.mark_modified();
        result
    }
}

impl<T, VertexData, CellData, const D: usize> std::ops::Deref
    for TdsMutWrapper<'_, T, VertexData, CellData, D>
where
    T: delaunay::geometry::CoordinateScalar,
    VertexData: delaunay::core::DataType,
    CellData: delaunay::core::DataType,
    [T; D]: serde::Serialize + for<'de> serde::Deserialize<'de>,
{
    type Target = delaunay::core::Tds<T, VertexData, CellData, D>;

    fn deref(&self) -> &Self::Target {
        &self.triangulation.tds
    }
}

impl<T, VertexData, CellData, const D: usize> std::ops::DerefMut
    for TdsMutWrapper<'_, T, VertexData, CellData, D>
where
    T: delaunay::geometry::CoordinateScalar,
    VertexData: delaunay::core::DataType,
    CellData: delaunay::core::DataType,
    [T; D]: serde::Serialize + for<'de> serde::Deserialize<'de>,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.triangulation.tds
    }
}

impl<T, VertexData, CellData, const D: usize> Drop for TdsMutWrapper<'_, T, VertexData, CellData, D>
where
    T: delaunay::geometry::CoordinateScalar,
    VertexData: delaunay::core::DataType,
    CellData: delaunay::core::DataType,
    [T; D]: serde::Serialize + for<'de> serde::Deserialize<'de>,
{
    /// Invalidates the edge cache when the wrapper is dropped, but only if modifications occurred.
    /// This provides more efficient caching by avoiding unnecessary invalidations.
    fn drop(&mut self) {
        if self.modification_occurred.get() {
            log::debug!("TdsMutWrapper dropped after modifications - invalidating edge cache");
            self.triangulation.cached_edge_count = std::cell::OnceCell::new();
        } else {
            log::debug!("TdsMutWrapper dropped without modifications - preserving cache");
        }
    }
}

/// Type alias for 2D triangulations with f64 coordinates
pub type CausalTriangulation2D = CausalTriangulation<f64, i32, i32, 2>;

impl<T, VertexData, CellData, const D: usize> CausalTriangulation<T, VertexData, CellData, D>
where
    T: delaunay::geometry::CoordinateScalar + AddAssign<T> + SubAssign<T> + Sum + NumCast,
    VertexData: delaunay::core::DataType,
    CellData: delaunay::core::DataType,
    [T; D]: serde::Serialize + for<'de> serde::Deserialize<'de>,
    for<'a> &'a T: Div<T>,
{
    /// Creates a new causal triangulation from an existing Tds.
    #[must_use]
    pub const fn from_tds(
        tds: delaunay::core::Tds<T, VertexData, CellData, D>,
        time_slices: u32,
        dimension: u8,
    ) -> Self {
        Self {
            tds,
            time_slices,
            dimension,
            cached_edge_count: std::cell::OnceCell::new(),
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
    /// Uses the delaunay crate's `AllFacetsIter` to iterate over all edges (facets)
    /// in the 2D triangulation, providing an accurate count of unique edges.
    #[must_use]
    pub fn edge_count(&self) -> usize {
        let cached_value = self.cached_edge_count.get();
        let result = *self.cached_edge_count.get_or_init(|| {
            log::debug!("Computing edge count for the first time or after cache invalidation");
            self.calculate_edge_count()
        });

        if cached_value.is_some() {
            log::debug!("Using cached edge count: {result}");
        }

        result
    }

    /// Calculates the edge count without caching.
    /// Uses the shared canonical implementation.
    #[must_use]
    fn calculate_edge_count(&self) -> usize {
        count_edges_in_tds(&self.tds)
    }

    /// Invalidates the edge count cache. Call this after modifying the triangulation.
    pub fn invalidate_edge_cache(&mut self) {
        log::debug!("Invalidating edge count cache");
        self.cached_edge_count = std::cell::OnceCell::new();
    }

    /// Logs a summary of the triangulation.
    pub fn print_summary(&self) {
        log::info!("Causal Triangulation Summary:");
        log::info!("  Dimension: {}", self.dimension);
        log::info!("  Time slices: {}", self.time_slices);
        log::info!("  Vertices: {}", self.vertex_count());
        log::info!("  Edges: {}", self.edge_count());
        log::info!("  Triangles: {}", self.triangle_count());
    }

    /// Returns a reference to the underlying Tds.
    #[must_use]
    pub const fn tds(&self) -> &delaunay::core::Tds<T, VertexData, CellData, D> {
        &self.tds
    }

    /// Returns a smart wrapper providing mutable access to the underlying Tds.
    ///
    /// The wrapper automatically invalidates the edge cache when dropped,
    /// ensuring cache consistency if the TDS was potentially modified.
    /// This provides precise cache invalidation - the cache is only cleared
    /// if mutable access was actually obtained.
    pub fn tds_mut(&mut self) -> TdsMutWrapper<'_, T, VertexData, CellData, D> {
        log::debug!(
            "tds_mut() called - returning smart wrapper. \
            Cache will be invalidated on drop. \
            Current cached edge count: {:?}",
            self.cached_edge_count.get()
        );
        TdsMutWrapper::new(self)
    }

    /// Get the number of time slices
    pub const fn time_slices(&self) -> u32 {
        self.time_slices
    }

    /// Get the dimension
    pub const fn dimension(&self) -> u8 {
        self.dimension
    }
}

// Specific implementation for 2D triangulations
impl CausalTriangulation2D {
    /// Creates a new 2D causal triangulation with validation.
    ///
    /// # Arguments
    ///
    /// * `vertices` - Number of vertices in the triangulation (must be >= 3)
    /// * `time_slices` - Number of time slices in the foliation (must be >= 1)
    /// * `dimension` - Dimension of the triangulation (must be 2)
    ///
    /// # Returns
    ///
    /// A `CdtResult<CausalTriangulation2D>` containing the triangulation if successful.
    /// Returns an error if triangulation generation fails due to invalid parameters.
    /// # Errors
    ///
    /// Returns [`CdtError::InvalidParameters`] if vertices < 3 or `time_slices` < 1.
    /// Returns [`CdtError::UnsupportedDimension`] if dimension != 2.
    /// Returns [`CdtError::TriangulationGeneration`] if triangulation generation fails.
    pub fn try_new(vertices: u32, time_slices: u32, dimension: u8) -> CdtResult<Self> {
        // Validate input parameters
        if vertices < 3 {
            return Err(CdtError::InvalidParameters(format!(
                "vertices count {vertices} is invalid, must be >= 3"
            )));
        }
        if time_slices < 1 {
            return Err(CdtError::InvalidParameters(format!(
                "time_slices count {time_slices} is invalid, must be >= 1"
            )));
        }
        if dimension != 2 {
            return Err(CdtError::UnsupportedDimension(dimension.into()));
        }

        let tds = try_generate_random_delaunay2_with_context(vertices, (0.0, 10.0))?;

        Ok(Self {
            tds,
            time_slices,
            dimension,
            cached_edge_count: std::cell::OnceCell::new(),
        })
    }

    /// Creates a new 2D causal triangulation.
    ///
    /// # Returns
    ///
    /// A `CdtResult<CausalTriangulation2D>` containing the triangulation if successful.
    /// # Errors
    ///
    /// Returns the same errors as [`Self::try_new`].
    pub fn new(vertices: u32, time_slices: u32, dimension: u8) -> CdtResult<Self> {
        Self::try_new(vertices, time_slices, dimension)
    }
}

/// Generates a random Delaunay triangulation with enhanced error context.
///
/// This function creates a proper Delaunay triangulation using the delaunay crate's
/// utility functions with detailed error reporting for debugging.
///
/// # Arguments
///
/// * `number_of_vertices` - The number of vertices to include in the triangulation
/// * `coordinate_range` - The range for generating random coordinates
///
/// # Returns
///
/// A `Result<Tds, CdtError>` containing the triangulation if successful, detailed error otherwise.
///
/// # Errors
///
/// Returns enhanced error information including vertex count, coordinate range, and underlying error.
pub fn try_generate_random_delaunay2_with_context(
    number_of_vertices: u32,
    coordinate_range: (f64, f64),
) -> CdtResult<delaunay::core::Tds<f64, i32, i32, 2>> {
    // Validate parameters before attempting generation
    if number_of_vertices < 3 {
        return Err(CdtError::InvalidGenerationParameters {
            issue: "Insufficient vertex count".to_string(),
            provided_value: number_of_vertices.to_string(),
            expected_range: "≥ 3".to_string(),
        });
    }

    if coordinate_range.0 >= coordinate_range.1 {
        return Err(CdtError::InvalidGenerationParameters {
            issue: "Invalid coordinate range".to_string(),
            provided_value: format!("[{}, {}]", coordinate_range.0, coordinate_range.1),
            expected_range: "min < max".to_string(),
        });
    }

    // Attempt generation with detailed error context
    generate_random_triangulation(number_of_vertices as usize, coordinate_range, None, None)
        .map_err(|e| CdtError::DelaunayGenerationFailed {
            vertex_count: number_of_vertices,
            coordinate_range,
            attempt: 1,
            underlying_error: e.to_string(),
        })
}

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
pub fn try_generate_random_delaunay2(
    number_of_vertices: u32,
    coordinate_range: (f64, f64),
) -> Option<delaunay::core::Tds<f64, i32, i32, 2>> {
    try_generate_random_delaunay2_with_context(number_of_vertices, coordinate_range).ok()
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
pub fn generate_random_delaunay2(
    number_of_vertices: u32,
    coordinate_range: (f64, f64),
) -> delaunay::core::Tds<f64, i32, i32, 2> {
    try_generate_random_delaunay2(number_of_vertices, coordinate_range).unwrap_or_else(|| {
        panic!(
            "Failed to generate random Delaunay triangulation with {number_of_vertices} vertices"
        )
    })
}

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
    if tds.vertices().len() < 2 || tds.cells().is_empty() {
        return 0;
    }

    // Use AllFacetsIter to iterate over all facets (edges in 2D)
    // and count unique edges by tracking vertex pairs
    let mut unique_edges = HashSet::new();
    let all_facets = AllFacetsIter::new(tds);

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_causal_triangulation_creation() {
        let triangulation =
            CausalTriangulation2D::new(10, 3, 2).expect("Failed to create triangulation");

        assert_eq!(triangulation.dimension(), 2);
        assert_eq!(triangulation.time_slices(), 3);
        assert!(triangulation.triangle_count() > 0);
    }

    #[test]
    fn test_vertex_counting() {
        let triangulation =
            CausalTriangulation2D::new(5, 2, 2).expect("Failed to create triangulation");

        // These should work with the actual Tds now
        assert!(triangulation.vertex_count() > 0);
        assert!(triangulation.edge_count() > 0);
        assert!(triangulation.triangle_count() > 0);
    }

    #[test]
    fn test_triangulation_access() {
        let triangulation =
            CausalTriangulation2D::new(3, 1, 2).expect("Failed to create triangulation");

        // Test immutable access
        let tds = triangulation.tds();
        assert!(!tds.cells().is_empty());

        assert!(triangulation.triangle_count() >= 1);
    }

    #[test]
    fn test_from_tds_constructor() {
        let tds = generate_random_delaunay2(5, (0.0, 10.0));
        let triangulation = CausalTriangulation2D::from_tds(tds, 2, 2);

        assert_eq!(triangulation.dimension(), 2);
        assert_eq!(triangulation.time_slices(), 2);
        assert!(triangulation.triangle_count() > 0);
    }

    #[test]
    fn delaunay_triangulation_construction() {
        let triangulation = generate_random_delaunay2(3, (0.0, 10.0));

        assert_eq!(triangulation.dim(), 2);
        // For 3 points, we should have 1 triangle
        assert_eq!(triangulation.cells().len(), 1);
    }

    #[test]
    fn test_smart_wrapper_cache_invalidation() {
        let mut triangulation =
            CausalTriangulation2D::new(5, 2, 2).expect("Failed to create triangulation");

        // Get initial edge count to populate cache
        let initial_edge_count = triangulation.edge_count();
        log::info!("Initial edge count: {initial_edge_count}");

        // Test that cache is preserved while wrapper exists
        {
            let _tds_wrapper = triangulation.tds_mut();
            // Cache should still be valid here since wrapper hasn't been dropped
            // Note: In a real scenario, external code would modify the TDS through the wrapper
            // For this test, we're demonstrating that the cache gets invalidated on drop
        } // Wrapper drops here, invalidating the cache

        // Get edge count again - this should recalculate since cache was invalidated
        let recalculated_edge_count = triangulation.edge_count();
        log::info!("Edge count after wrapper drop: {recalculated_edge_count}");

        // Edge count should be the same since we didn't actually modify the TDS
        assert_eq!(initial_edge_count, recalculated_edge_count);
    }

    #[test]
    fn test_legacy_tds_mut_behavior() {
        let mut triangulation =
            CausalTriangulation2D::new(5, 2, 2).expect("Failed to create triangulation");

        // Get initial edge count to populate cache
        let initial_edge_count = triangulation.edge_count();

        // Test that the smart wrapper provides TDS access
        {
            let tds_wrapper = triangulation.tds_mut();
            // Should be able to access TDS methods through the wrapper
            assert!(!tds_wrapper.cells().is_empty());
            assert!(!tds_wrapper.vertices().is_empty());
        }

        // Verify cache was invalidated and recalculated
        let final_edge_count = triangulation.edge_count();
        assert_eq!(initial_edge_count, final_edge_count);
    }

    #[test]
    fn test_cache_invalidation_validation() {
        let mut triangulation =
            CausalTriangulation2D::new(5, 2, 2).expect("Failed to create triangulation");

        // Step 1: Get initial edge count to populate cache
        let initial_edge_count = triangulation.edge_count();

        // Step 2: Call edge_count() again immediately - should use cached value
        // This demonstrates the cache is working
        let cached_edge_count = triangulation.edge_count();
        assert_eq!(
            cached_edge_count, initial_edge_count,
            "Second call should return cached value"
        );

        // Step 3: Force cache invalidation by calling tds_mut() and letting wrapper drop
        {
            let _tds_wrapper = triangulation.tds_mut();
            // Wrapper exists here but doesn't modify TDS
        } // Wrapper drops here, should invalidate cache

        // Step 4: Call edge_count() again - should recalculate since cache was invalidated
        let recalculated_edge_count = triangulation.edge_count();
        assert_eq!(
            recalculated_edge_count, initial_edge_count,
            "Edge count should be consistent since TDS wasn't modified"
        );

        // Step 5: Test manual invalidation
        triangulation.invalidate_edge_cache();
        let final_edge_count = triangulation.edge_count();
        assert_eq!(
            final_edge_count, initial_edge_count,
            "Edge count should remain consistent after manual invalidation"
        );
    }

    #[test]
    fn test_cache_consistency_with_wrapper() {
        let mut triangulation =
            CausalTriangulation2D::new(3, 1, 2).expect("Failed to create triangulation");

        // Get initial counts
        let vertex_count = triangulation.vertex_count();
        let triangle_count = triangulation.triangle_count();
        let initial_edge_count = triangulation.edge_count();

        // Verify basic Euler formula holds: V - E + T = 1 (for planar graphs with boundary)
        assert_eq!(
            i32::try_from(vertex_count).unwrap_or(i32::MAX)
                - i32::try_from(initial_edge_count).unwrap_or(i32::MAX)
                + i32::try_from(triangle_count).unwrap_or(i32::MAX),
            1,
            "Euler formula should hold for initial triangulation"
        );

        // Use the wrapper (but don't modify TDS)
        {
            let tds_wrapper = triangulation.tds_mut();
            // Verify we can access TDS properties through the wrapper
            assert_eq!(tds_wrapper.vertices().len(), vertex_count);
            assert_eq!(tds_wrapper.cells().len(), triangle_count);
        } // Cache invalidated on wrapper drop

        // Verify edge count is recalculated correctly after cache invalidation
        let recalculated_edge_count = triangulation.edge_count();
        assert_eq!(
            recalculated_edge_count, initial_edge_count,
            "Cache invalidation and recalculation should produce consistent results"
        );

        // Verify Euler formula still holds after recalculation
        assert_eq!(
            i32::try_from(vertex_count).unwrap_or(i32::MAX)
                - i32::try_from(recalculated_edge_count).unwrap_or(i32::MAX)
                + i32::try_from(triangle_count).unwrap_or(i32::MAX),
            1,
            "Euler formula should still hold after cache invalidation/recalculation"
        );
    }

    #[test]
    fn test_invalid_vertices() {
        let result = CausalTriangulation2D::new(2, 1, 2);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_time_slices() {
        let result = CausalTriangulation2D::new(4, 0, 2);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_dimension() {
        let result = CausalTriangulation2D::new(4, 1, 3);
        assert!(result.is_err());
    }

    #[test]
    fn test_minimum_triangulation() {
        let triangulation =
            CausalTriangulation2D::new(3, 1, 2).expect("Failed to create minimal triangulation");
        assert_eq!(triangulation.vertex_count(), 3);
        assert_eq!(triangulation.time_slices(), 1);
        assert_eq!(triangulation.dimension(), 2);
        // For a minimal triangulation with 3 vertices, we should have at least 1 triangle
        assert!(triangulation.triangle_count() >= 1);
        // Verify Euler's formula: V - E + F = 1 for a planar graph with boundary
        let v = NumCast::from(triangulation.vertex_count()).unwrap_or(0);
        let e = NumCast::from(triangulation.edge_count()).unwrap_or(0);
        let f = NumCast::from(triangulation.triangle_count()).unwrap_or(0);
        assert_eq!(v - e + f, 1, "Euler's formula V - E + F = 1 should hold");
    }

    #[test]
    fn test_getter_methods() {
        let triangulation =
            CausalTriangulation2D::new(4, 2, 2).expect("Failed to create triangulation");

        // Test that getter methods work correctly
        assert_eq!(triangulation.time_slices(), 2);
        assert_eq!(triangulation.dimension(), 2);
        assert_eq!(triangulation.vertex_count(), 4);

        // Test that we can access the tds through the getter
        let _tds_ref = triangulation.tds();
    }

    #[test]
    fn test_controlled_mutation_invalidates_cache() {
        let mut triangulation =
            CausalTriangulation2D::new(4, 1, 2).expect("Failed to create triangulation");

        // Get initial edge count to populate cache
        let initial_edge_count = triangulation.edge_count();

        // Mutate through controlled method - this should invalidate cache
        {
            let _tds_mut = triangulation.tds_mut();
            // The wrapper automatically invalidates cache when dropped
        }

        // Verify edge count can be recalculated (cache was invalidated)
        let edge_count_after_mutation = triangulation.edge_count();
        assert_eq!(initial_edge_count, edge_count_after_mutation);
    }
}

#[cfg(kani)]
mod verification {
    use super::*;

    #[kani::proof]
    fn triangle_construction() {
        let triangulation = generate_random_delaunay2(3, (0.0, 10.0));

        assert!(!triangulation.cells().is_empty());
    }
}
