use rand::random;

/// Generates a random floating-point number between 0.0 and 1.0.
///
/// # Returns
///
/// A random `f64` value in the range [0.0, 1.0).
#[must_use]
pub fn generate_random_float() -> f64 {
    random::<f64>()
}

/// Generates a Delaunay triangulation with optional seed for deterministic testing.
///
/// # Errors
///
/// Returns enhanced error information including vertex count, coordinate range, and underlying error.
pub fn generate_delaunay2_with_context(
    number_of_vertices: u32,
    coordinate_range: (f64, f64),
    seed: Option<u64>,
) -> crate::errors::CdtResult<delaunay::core::Tds<f64, i32, i32, 2>> {
    use crate::errors::CdtError;

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

    // Generate triangulation with or without seed
    delaunay::geometry::util::generate_random_triangulation(
        number_of_vertices as usize,
        coordinate_range,
        None,
        seed,
    )
    .map_err(|e| CdtError::DelaunayGenerationFailed {
        vertex_count: number_of_vertices,
        coordinate_range,
        attempt: 1,
        underlying_error: e.to_string(),
    })
}

/// Generates a random Delaunay triangulation.
///
/// # Panics
///
/// Panics if triangulation generation fails due to invalid parameters or coordinate generation errors.
#[must_use]
pub fn generate_random_delaunay2(
    number_of_vertices: u32,
    coordinate_range: (f64, f64),
) -> delaunay::core::Tds<f64, i32, i32, 2> {
    generate_delaunay2_with_context(number_of_vertices, coordinate_range, None)
        .unwrap_or_else(|_| {
            panic!(
                "Failed to generate random Delaunay triangulation with {number_of_vertices} vertices"
            )
        })
}

/// Generates a seeded Delaunay triangulation for deterministic testing.
///
/// # Panics
///
/// Panics if triangulation generation fails due to invalid parameters or coordinate generation errors.
#[must_use]
pub fn generate_seeded_delaunay2(
    number_of_vertices: u32,
    coordinate_range: (f64, f64),
    seed: u64,
) -> delaunay::core::Tds<f64, i32, i32, 2> {
    generate_delaunay2_with_context(number_of_vertices, coordinate_range, Some(seed))
        .unwrap_or_else(|_| {
            panic!(
                "Failed to generate seeded Delaunay triangulation with {number_of_vertices} vertices and seed {seed}"
            )
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_generate_random_float() {
        let result = generate_random_float();
        assert!(result > 0.0);
        assert!(result < 1.0);
    }
}
