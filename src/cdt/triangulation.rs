//! CDT triangulation wrapper - backend-agnostic.
//!
//! This module provides CDT-specific triangulation data structures that work
//! with any geometry backend implementing the trait interfaces.

use crate::errors::CdtResult;
use crate::geometry::traits::TriangulationMut;
use std::time::Instant;

/// CDT-specific triangulation wrapper - completely geometry-agnostic
#[derive(Debug)]
pub struct CdtTriangulation<B: TriangulationMut> {
    geometry: B,
    metadata: CdtMetadata,
    cache: GeometryCache,
}

/// CDT-specific metadata
#[derive(Debug, Clone)]
pub struct CdtMetadata {
    /// Number of time slices in the CDT foliation
    pub time_slices: u32,
    /// Dimensionality of the spacetime
    pub dimension: u8,
    /// Time when this triangulation was created
    pub creation_time: Instant,
    /// Time of last modification
    pub last_modified: Instant,
    /// Count of modifications made to the triangulation
    pub modification_count: u64,
    /// History of simulation events
    pub simulation_history: Vec<SimulationEvent>,
}

/// Cached geometry measurements
#[derive(Debug, Clone, Default)]
struct GeometryCache {
    edge_count: Option<CachedValue<usize>>,
    euler_char: Option<CachedValue<i32>>,
    #[allow(dead_code)]
    topology_hash: Option<CachedValue<u64>>,
}

#[derive(Debug, Clone)]
struct CachedValue<T> {
    value: T,
    #[allow(dead_code)]
    computed_at: Instant,
    modification_count: u64,
}

/// Events in simulation history
#[derive(Debug, Clone)]
pub enum SimulationEvent {
    /// Triangulation was created
    Created {
        /// Initial number of vertices
        vertex_count: usize,
        /// Number of time slices
        time_slices: u32,
    },
    /// An ergodic move was attempted
    MoveAttempted {
        /// Type of move attempted
        move_type: String,
        /// Simulation step number
        step: u64,
    },
    /// An ergodic move was accepted
    MoveAccepted {
        /// Type of move accepted
        move_type: String,
        /// Simulation step number
        step: u64,
        /// Change in action from this move
        action_change: f64,
    },
    /// A measurement was taken
    MeasurementTaken {
        /// Simulation step number
        step: u64,
        /// Action value measured
        action: f64,
    },
}

impl<B: TriangulationMut> CdtTriangulation<B> {
    /// Create new CDT triangulation
    pub fn new(geometry: B, time_slices: u32, dimension: u8) -> Self {
        let vertex_count = geometry.vertex_count();
        let creation_event = SimulationEvent::Created {
            vertex_count,
            time_slices,
        };

        Self {
            geometry,
            metadata: CdtMetadata {
                time_slices,
                dimension,
                creation_time: Instant::now(),
                last_modified: Instant::now(),
                modification_count: 0,
                simulation_history: vec![creation_event],
            },
            cache: GeometryCache::default(),
        }
    }

    /// Get immutable reference to underlying geometry
    #[must_use]
    pub const fn geometry(&self) -> &B {
        &self.geometry
    }

    /// Get mutable reference with automatic cache invalidation
    pub fn geometry_mut(&mut self) -> CdtGeometryMut<'_, B> {
        self.invalidate_cache();
        self.metadata.last_modified = Instant::now();
        self.metadata.modification_count += 1;
        CdtGeometryMut {
            geometry: &mut self.geometry,
            metadata: &mut self.metadata,
        }
    }

    /// CDT-specific operations
    pub fn vertex_count(&self) -> usize {
        self.geometry.vertex_count()
    }

    /// Get the number of faces in the triangulation
    pub fn face_count(&self) -> usize {
        self.geometry.face_count()
    }

    /// Get the number of time slices in the CDT foliation
    #[must_use]
    pub const fn time_slices(&self) -> u32 {
        self.metadata.time_slices
    }

    /// Get the dimensionality of the spacetime
    #[must_use]
    pub const fn dimension(&self) -> u8 {
        self.metadata.dimension
    }

    /// Cached edge count with automatic invalidation
    pub fn edge_count(&self) -> usize {
        if let Some(cached) = &self.cache.edge_count
            && cached.modification_count == self.metadata.modification_count
        {
            return cached.value;
        }

        self.geometry.edge_count()
    }

    /// Force cache update
    pub fn refresh_cache(&mut self) {
        let now = Instant::now();
        let mod_count = self.metadata.modification_count;

        self.cache.edge_count = Some(CachedValue {
            value: self.geometry.edge_count(),
            computed_at: now,
            modification_count: mod_count,
        });

        self.cache.euler_char = Some(CachedValue {
            value: self.geometry.euler_characteristic(),
            computed_at: now,
            modification_count: mod_count,
        });
    }

    /// Validate CDT properties
    ///
    /// # Errors
    /// Returns error if the triangulation is invalid
    pub fn validate_cdt_properties(&self) -> CdtResult<()> {
        // TODO: Implement full CDT property validation
        // - Check topology (Euler characteristic)
        // - Check causality constraints
        // - Check foliation consistency

        if !self.geometry.is_valid() {
            return Err(crate::errors::CdtError::InvalidParameters(
                "Invalid geometry: triangulation is not valid".to_string(),
            ));
        }

        Ok(())
    }

    fn invalidate_cache(&mut self) {
        self.cache = GeometryCache::default();
    }
}

/// Smart wrapper for mutable geometry access
pub struct CdtGeometryMut<'a, B: TriangulationMut> {
    geometry: &'a mut B,
    metadata: &'a mut CdtMetadata,
}

impl<B: TriangulationMut> CdtGeometryMut<'_, B> {
    /// Record a simulation event
    pub fn record_event(&mut self, event: SimulationEvent) {
        self.metadata.simulation_history.push(event);
    }

    /// Get mutable reference to geometry
    pub const fn geometry_mut(&mut self) -> &mut B {
        self.geometry
    }
}

impl<B: TriangulationMut> std::ops::Deref for CdtGeometryMut<'_, B> {
    type Target = B;
    fn deref(&self) -> &Self::Target {
        self.geometry
    }
}

impl<B: TriangulationMut> std::ops::DerefMut for CdtGeometryMut<'_, B> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.geometry
    }
}

// Factory functions for creating CdtTriangulations with different backends
impl CdtTriangulation<crate::geometry::backends::delaunay::DelaunayBackend2D> {
    /// Create a new CDT triangulation with Delaunay backend.
    ///
    /// This is the recommended way to create triangulations with the new trait-based system.
    ///
    /// # Errors
    /// Returns error if triangulation generation fails
    pub fn new_with_delaunay(
        vertices: u32,
        time_slices: u32,
        dimension: u8,
    ) -> crate::errors::CdtResult<Self> {
        use crate::geometry::backends::delaunay::DelaunayBackend;
        use crate::triangulations::triangulation::try_generate_random_delaunay2_with_context;

        if dimension != 2 {
            return Err(crate::errors::CdtError::UnsupportedDimension(
                dimension.into(),
            ));
        }

        let tds = try_generate_random_delaunay2_with_context(vertices, (0.0, 10.0))?;
        let backend = DelaunayBackend::from_tds(tds);

        Ok(Self::new(backend, time_slices, dimension))
    }

    /// Convert from legacy `CausalTriangulation` to new `CdtTriangulation`.
    ///
    /// This is a migration helper to move from the deprecated direct Tds structure
    /// to the new trait-based design.
    #[allow(deprecated)]
    pub fn from_causal_triangulation(
        old: &crate::triangulations::triangulation::CausalTriangulation2D,
    ) -> Self {
        use crate::geometry::backends::delaunay::DelaunayBackend;

        let time_slices = old.time_slices();
        let dimension = old.dimension();

        // Clone the TDS from the old structure
        let tds = old.tds().clone();
        let backend = DelaunayBackend::from_tds(tds);

        Self::new(backend, time_slices, dimension)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::traits::TriangulationQuery;

    #[test]
    fn test_new_with_delaunay() {
        let triangulation = CdtTriangulation::new_with_delaunay(10, 3, 2)
            .expect("Failed to create triangulation with delaunay backend");

        assert_eq!(triangulation.dimension(), 2);
        assert_eq!(triangulation.time_slices(), 3);
        assert!(triangulation.vertex_count() > 0);
        assert!(triangulation.edge_count() > 0);
        assert!(triangulation.face_count() > 0);
    }

    #[test]
    #[allow(deprecated)]
    fn test_from_causal_triangulation() {
        use crate::triangulations::triangulation::CausalTriangulation2D;

        // Create old-style triangulation
        let old_triangulation =
            CausalTriangulation2D::new(5, 2, 2).expect("Failed to create old triangulation");

        let old_time_slices = old_triangulation.time_slices();
        let old_dimension = old_triangulation.dimension();
        let old_vertex_count = old_triangulation.vertex_count();

        // Convert to new structure
        let new_triangulation = CdtTriangulation::from_causal_triangulation(&old_triangulation);

        // Verify that data was preserved
        assert_eq!(new_triangulation.time_slices(), old_time_slices);
        assert_eq!(new_triangulation.dimension(), old_dimension);
        assert_eq!(new_triangulation.vertex_count(), old_vertex_count);
    }

    #[test]
    fn test_invalid_dimension() {
        let result = CdtTriangulation::new_with_delaunay(10, 3, 3);
        assert!(result.is_err());
    }

    #[test]
    fn test_geometry_access() {
        let triangulation =
            CdtTriangulation::new_with_delaunay(5, 2, 2).expect("Failed to create triangulation");

        // Test immutable access
        let geometry = triangulation.geometry();
        assert!(geometry.vertex_count() > 0);
        assert!(geometry.is_valid());
    }

    #[test]
    fn test_geometry_mut_with_cache() {
        let mut triangulation =
            CdtTriangulation::new_with_delaunay(5, 2, 2).expect("Failed to create triangulation");

        // Get initial edge count (populates cache)
        let initial_edge_count = triangulation.edge_count();
        assert!(initial_edge_count > 0);

        // Get mutable access - this should invalidate cache
        {
            let mut geometry_mut = triangulation.geometry_mut();
            // Just access it, don't modify
            let _ = geometry_mut.geometry_mut();
        }

        // Cache should have been invalidated but recalculated value should be same
        let recalculated_edge_count = triangulation.edge_count();
        assert_eq!(initial_edge_count, recalculated_edge_count);
    }

    #[test]
    fn test_euler_characteristic() {
        use crate::geometry::traits::TriangulationQuery;

        let triangulation =
            CdtTriangulation::new_with_delaunay(5, 2, 2).expect("Failed to create triangulation");

        let euler_char = triangulation.geometry().euler_characteristic();

        // For a planar triangulation with boundary, Euler characteristic should be 1
        assert_eq!(
            euler_char, 1,
            "Euler characteristic V - E + F should equal 1"
        );
    }
}

// TODO: Add serialization/deserialization support
// TODO: Add visualization hooks
