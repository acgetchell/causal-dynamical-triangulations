use crate::util::generate_random_float;

#[must_use]
/// Generates a random Delaunay triangulation with the specified number of vertices.
///
/// This is currently a placeholder implementation that creates a simple triangulation
/// by connecting consecutive points. A proper Delaunay triangulation algorithm
/// will be implemented once the delaunay crate API is integrated.
///
/// # Arguments
///
/// * `number_of_vertices` - The number of vertices to include in the triangulation
///
/// # Returns
///
/// A vector of triangles, where each triangle is represented as a vector of vertex indices.
pub fn generate_random_delaunay2(number_of_vertices: u32) -> Vec<Vec<usize>> {
    let mut points = Vec::new();

    for _n in 0..number_of_vertices {
        let point = generate_random_vertex(10.0);
        points.push(point);
    }

    // For now, create a simple triangulation by connecting points
    // This is a placeholder implementation until we figure out the delaunay crate API
    let mut triangulation = Vec::new();

    if points.len() >= 3 {
        // Create triangles by connecting consecutive points
        for i in 0..(points.len().saturating_sub(2)) {
            triangulation.push(vec![i, i + 1, i + 2]);
        }
    }

    triangulation
}

fn generate_random_vertex(scale: f64) -> (f64, f64) {
    let x = generate_random_float() * scale;
    let y = generate_random_float() * scale;
    (x, y)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn random_point_construction() {
        let scale = 10.0;
        let point = generate_random_vertex(scale);

        assert!(point.0 >= 0.0);
        assert!(point.0 < scale);
        assert!(point.1 >= 0.0);
        assert!(point.1 < scale);
    }

    #[test]
    fn delaunay_triangulation_construction() {
        let triangulation = generate_random_delaunay2(3);

        assert!(!triangulation.is_empty());
        // For 3 points, we should have 1 triangle
        assert_eq!(triangulation.len(), 1);
        // Each triangle should have 3 vertices
        assert_eq!(triangulation[0].len(), 3);
    }
}

#[cfg(kani)]
#[cfg(not(tarpaulin_include))]
mod verification {

    use super::*;

    #[kani::proof]
    fn triangle_construction() {
        let triangulation = generate_random_delaunay2(3);

        assert!(!triangulation.is_empty());
        assert!(triangulation.len() >= 1);
        // Each triangle should have exactly 3 vertices
        for triangle in &triangulation {
            assert_eq!(triangle.len(), 3);
        }
    }
}
