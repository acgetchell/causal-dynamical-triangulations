# cdt-rs

[![CI](https://github.com/acgetchell/cdt/actions/workflows/ci.yml/badge.svg)](https://github.com/acgetchell/cdt/actions/workflows/ci.yml)
[![rust-clippy analyze](https://github.com/acgetchell/cdt-rs/actions/workflows/rust-clippy.yml/badge.svg)](https://github.com/acgetchell/cdt-rs/actions/workflows/rust-clippy.yml)
[![Codecov](https://codecov.io/gh/acgetchell/cdt-rs/graph/badge.svg?token=CsbOJBypGC)](https://codecov.io/gh/acgetchell/cdt-rs)
[![Kani CI](https://github.com/acgetchell/cdt-rs/actions/workflows/kani.yml/badge.svg)](https://github.com/acgetchell/cdt-rs/actions/workflows/kani.yml)
[![Audit dependencies](https://github.com/acgetchell/cdt-rs/actions/workflows/audit.yml/badge.svg)](https://github.com/acgetchell/cdt-rs/actions/workflows/audit.yml)

Causal Dynamical Triangulations using Constrained Delaunay Triangulations in Rust

## Introduction

For an introduction to Causal Dynamical Triangulations, see [this paper](https://arxiv.org/abs/hep-th/0105267).

## Roadmap

- [x] Use [Kani] to verify correctness
- [x] Use an existing Rust Delaunay triangulation library (e.g. [delaunay])
- [x] 2D Delaunay triangulation
- [ ] 1+1 foliation
- [ ] 2D ergodic moves
- [ ] 2D Metropolis-Hastings
- [ ] Diffusion-accelerated MCMC?
- [ ] Visual output
- [ ] 3D Delaunay triangulation
- [ ] 2+1 foliation
- [ ] 3D ergodic moves
- [ ] 3D Metropolis-Hastings
- [ ] Visual output
- [ ] 4D Delaunay triangulation
- [ ] 3+1 foliation
- [ ] 4D ergodic moves
- [ ] 4D Metropolis-Hastings
- [ ] Initialize masses in 3D with Constrained Delaunay Triangulation
- [ ] Initialize masses in 4D with Constrained Delaunay Triangulation
- [ ] Shortest path
- [ ] Geodesic distance
- [ ] Einstein tensor
- [ ] Quantize spacetime!

## Issues

- [x] Using [delaunay] crate for 2D Delaunay triangulation

[delaunay]: https://crates.io/crates/delaunay
[Kani]: https://model-checking.github.io/kani/install-guide.html
