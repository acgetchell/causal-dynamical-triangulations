// Temporary script to find good seeds for triangulation generation
use causal_dynamical_triangulations::cdt::triangulation::CdtTriangulation;

fn test_seed_with_params(seed: u64, vertices: u32, timeslices: u32) -> Option<(i32, usize, usize, usize)> {
    // We'll need to modify the triangulation generation to accept seeds
    // For now, let's just test with the current random generation
    match CdtTriangulation::from_random_points(vertices, timeslices, 2) {
        Ok(tri) => {
            let v = tri.vertex_count() as i32;
            let e = tri.edge_count() as i32;
            let f = tri.face_count() as i32;
            let euler = v - e + f;
            
            println!("Vertices: {}, Edges: {}, Faces: {}, Euler: {}", v, e, f, euler);
            
            if (0..=2).contains(&euler) {
                Some((euler, v as usize, e as usize, f as usize))
            } else {
                None
            }
        },
        Err(_) => None
    }
}

fn main() {
    println!("Testing different triangulation generations to find good patterns...");
    
    let mut good_cases = Vec::new();
    
    for i in 0..50 {
        println!("\n=== Attempt {} ===", i + 1);
        if let Some(result) = test_seed_with_params(i, 7, 3) {
            good_cases.push((i, result));
            if good_cases.len() >= 5 {
                break;
            }
        }
    }
    
    println!("\n=== GOOD CASES FOUND ===");
    for (attempt, (euler, v, e, f)) in good_cases {
        println!("Attempt {}: V={}, E={}, F={}, Euler={}", attempt, v, e, f, euler);
    }
}
