src/
├── lib.rs                           # Clean public API, feature-gated exports
├── main.rs                          # CLI entry point (unchanged)
├── errors.rs                        # Comprehensive, backend-agnostic errors
├── util.rs                          # Utilities (unchanged)
│
├── geometry/                        # ← NEW: Complete geometry abstraction
│   ├── mod.rs                       # Geometry module exports
│   ├── traits.rs                    # Core geometry traits (GeometryBackend, etc.)
│   ├── mesh.rs                      # CDT-agnostic mesh data structures  
│   ├── operations.rs                # High-level triangulation operations
│   └── backends/                    # ← NEW: Pluggable backends
│       ├── mod.rs                   # Backend registry and selection
│       ├── delaunay.rs              # Delaunay crate wrapper (isolated)
│       ├── mock.rs                  # Mock backend for testing
│       └── registry.rs              # Runtime backend selection
│
├── cdt/                             # ← RESTRUCTURED: Pure CDT logic
│   ├── mod.rs                       # CDT module exports
│   ├── triangulation.rs             # CdtTriangulation<B: Backend> (generic)
│   ├── action.rs                    # Action calc (backend-agnostic)
│   ├── metropolis.rs                # Metropolis (works with traits)
│   ├── ergodic_moves.rs             # Ergodic moves (trait-based)
│   └── simulation.rs                # ← NEW: High-level simulation orchestration
│
├── config/                          # ← NEW: Configuration management
│   ├── mod.rs                       # Config exports
│   ├── simulation.rs                # Simulation parameters
│   ├── physics.rs                   # Physics constants and validation
│   └── backends.rs                  # Backend-specific configurations
│
├── observables/                     # ← NEW: Measurement and analysis
│   ├── mod.rs                       # Observables exports
│   ├── measurements.rs              # Data collection during simulation
│   ├── analysis.rs                  # Statistical analysis tools
│   └── export.rs                    # Data export (JSON, CSV, etc.)
│
└── cli/                            # ← NEW: CLI module (feature-gated)
    ├── mod.rs                      # CLI exports
    ├── args.rs                     # Argument parsing
    ├── commands.rs                 # Command implementations
    └── output.rs                   # Output formatting
