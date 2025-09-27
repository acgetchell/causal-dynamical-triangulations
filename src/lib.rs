#![allow(clippy::multiple_crate_versions)]
// #![warn(missing_docs)]

//! Using <https://crates.io/crates/clap> for command line arguments
//! Using <https://crates.io/crates/delaunay> for 2D Delaunay triangulation
use clap::Parser;

mod triangulations {
    pub mod delaunay_triangulations;
}

use triangulations::delaunay_triangulations::generate_random_delaunay2;

/// Contains utility functions for the `cdt-rs` crate.
pub mod util;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
/// Configuration options for the `cdt-rs` crate.
pub struct Config {
    /// Dimensionality of the triangulation
    #[arg(short, long, value_parser = clap::value_parser!(u32).range(2..4))]
    dimension: Option<u32>,

    /// Number of vertices
    #[arg(short, long, value_parser = clap::value_parser!(u32).range(3..))]
    vertices: u32,

    /// Number of timeslices
    #[arg(short, long, value_parser = clap::value_parser!(u32).range(1..))]
    timeslices: u32,
}

impl Config {
    /// Builds a new instance of `Config`.
    #[must_use]
    pub fn build() -> Self {
        Self::parse()
    }
}

/// Runs the triangulation with the given configuration.
#[must_use]
pub fn run(config: &Config) -> Vec<Vec<usize>> {
    let vertices = config.vertices;
    let timeslices = config.timeslices;

    if config.dimension.is_some_and(|d| d != 2) {
        eprintln!("Only 2D triangulations are supported right now.");
        std::process::exit(1);
    }

    println!("Dimensionality: {}", config.dimension.unwrap_or(2));
    println!("Number of vertices: {vertices}");
    println!("Number of timeslices: {timeslices}");

    let triangulation = generate_random_delaunay2(vertices);

    println!("Number of triangles: {}", triangulation.len());

    // Print triangle indices
    for (i, triangle) in triangulation.iter().enumerate() {
        println!("Triangle {i}: vertices {triangle:?}");
    }

    triangulation
}

#[cfg(test)]
mod lib_tests {
    use super::*;
    #[test]
    fn test_run() {
        let config = Config {
            dimension: Some(2),
            vertices: 32,
            timeslices: 3,
        };
        assert!(config.dimension.is_some());
        let triangulation = run(&config);
        assert!(!triangulation.is_empty());
    }

    #[test]
    fn triangulation_contains_triangles() {
        let config = Config {
            dimension: Some(2),
            vertices: 32,
            timeslices: 3,
        };
        let triangulation = run(&config);
        // Check that we have some triangles
        assert!(!triangulation.is_empty());
    }
}
