# Ergodic Moves

Ergodic moves are the local Monte Carlo updates that allow the triangulation to explore the space of geometries. This module implements the standard ergodic moves for 2D Causal Dynamical Triangulations (see `src/cdt/ergodic_moves.rs`).

## Types

### `MoveType`

Enumerates the available move types:

- `Move22` — (2,2) move: flip the shared edge between two triangles, preserving vertex count
- `Move13Add` — (1,3) move: insert a new vertex by subdividing one triangle into three
- `Move31Remove` — (3,1) move: remove a vertex by merging three triangles into one
- `EdgeFlip` — standard Delaunay edge flip maintaining the Delaunay property while potentially modifying causal structure

### `MoveResult`

Returned by each `attempt_*` method:

- `Success` — move was applied
- `CausalityViolation` — rejected because the move would break causal layering
- `GeometricViolation` — rejected because the resulting triangulation would be geometrically invalid
- `Rejected(CdtError)` — rejected for another reason, with details

### `MoveStatistics`

Tracks per-move-type attempt and acceptance counts. Fields follow the pattern `moves_NN_attempted` / `moves_NN_accepted` for (2,2), (1,3), and (3,1) moves, plus `edge_flips_attempted` / `edge_flips_accepted`.

Key methods:

- `record_attempt(MoveType)` — increment the attempt counter
- `record_success(MoveType)` — increment the acceptance counter
- `acceptance_rate(MoveType) -> f64` — ratio for a single move type
- `total_acceptance_rate() -> f64` — ratio across all move types

### `ErgodicsSystem`

Owns a `MoveStatistics` instance and a thread-local RNG. Public API:

- `new()` / `Default::default()` — construct
- `select_random_move() -> MoveType` — sample uniformly from all four move types
- `attempt_22_move(triangulation) -> MoveResult`
- `attempt_13_move(triangulation) -> MoveResult`
- `attempt_31_move(triangulation) -> MoveResult`
- `attempt_edge_flip(triangulation) -> MoveResult`
- `attempt_random_move(triangulation) -> MoveResult` — delegates to one of the above

> **Note**: All `attempt_*` methods are currently placeholder implementations that simulate realistic acceptance rates. Full integration with the `delaunay` crate's `Tds` type is planned for a future release.

## Architecture

Move validation follows a two-layer design:

- **`delaunay` crate** — pure geometric operations (bistellar flips, edge flips) with no physics constraints
- **CDT crate** — wraps geometric operations with causality and time-slice validation

When the `delaunay` crate exposes `try_edge_flip` / `try_bistellar_flip`, the placeholder bodies will be replaced with calls to those methods guarded by CDT-specific pre-checks.

## Planned Work

- [ ] Implement `try_edge_flip()` in `delaunay` for (2,2) moves
- [ ] Implement `try_bistellar_flip()` in `delaunay` for (1,3)/(3,1) moves
- [ ] Replace placeholder bodies with real geometric operations
- [ ] Add causality and time-slice constraint validation
