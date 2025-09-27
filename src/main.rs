//! Causal Dynamical Triangulations binary executable.
//!
//! This is the main entry point for the CDT-RS application that creates
//! and runs causal dynamical triangulations simulations.

use causal_dynamical_triangulations::{Config, run};

fn main() {
    let config = Config::build();
    let _results = run(&config);
}
