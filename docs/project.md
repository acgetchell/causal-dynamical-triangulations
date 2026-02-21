# Project Structure

```
src/
├── lib.rs             # Public API and module exports
├── main.rs            # CLI entry point
├── errors.rs          # Error types (CdtError)
├── util.rs            # Utility functions
├── config.rs          # Simulation configuration
├── geometry/          # Geometry abstraction layer
│   ├── traits.rs      # Core geometry traits (GeometryBackend, etc.)
│   ├── mesh.rs        # CDT-agnostic mesh data structures
│   ├── operations.rs  # High-level triangulation operations
│   └── backends/      # Pluggable geometry backends
│       ├── delaunay.rs # Delaunay crate wrapper
│       └── mock.rs    # Mock backend for testing
└── cdt/               # CDT physics and Monte Carlo logic
    ├── triangulation.rs # CdtTriangulation core type
    ├── action.rs        # Regge action calculation
    ├── metropolis.rs    # Metropolis-Hastings algorithm
    └── ergodic_moves.rs # Ergodic moves (2,2), (1,3), (3,1)
```
