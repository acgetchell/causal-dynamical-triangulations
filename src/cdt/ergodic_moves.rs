//! Ergodic moves for 2D Causal Dynamical Triangulations.
//!
//! This module implements the standard ergodic moves used in CDT:
//! - (2,2) moves: Flip edge between two triangles
//! - (1,3) moves: Add/remove vertex with triangle subdivision
//! - Edge flips: Standard Delaunay edge flips maintaining causality

use crate::util::generate_random_float;
use rand::Rng;

/// Types of ergodic moves available in 2D CDT.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MoveType {
    /// (2,2) move: Flip edge between two triangles
    Move22,
    /// (1,3) move: Add vertex by subdividing triangle
    Move13Add,
    /// (3,1) move: Remove vertex by merging triangles
    Move31Remove,
    /// Edge flip: Standard Delaunay edge flip
    EdgeFlip,
}

/// Result of attempting an ergodic move.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MoveResult {
    /// Move was successfully applied
    Success,
    /// Move was rejected due to causality constraints
    CausalityViolation,
    /// Move was rejected due to geometric constraints
    GeometricViolation,
    /// Move was rejected for other reasons
    Rejected(String),
}

/// Statistics tracking for ergodic moves.
#[derive(Debug, Default)]
pub struct MoveStatistics {
    /// Number of (2,2) moves attempted
    pub moves_22_attempted: u64,
    /// Number of (2,2) moves accepted
    pub moves_22_accepted: u64,
    /// Number of (1,3) moves attempted
    pub moves_13_attempted: u64,
    /// Number of (1,3) moves accepted
    pub moves_13_accepted: u64,
    /// Number of (3,1) moves attempted
    pub moves_31_attempted: u64,
    /// Number of (3,1) moves accepted
    pub moves_31_accepted: u64,
    /// Number of edge flips attempted
    pub edge_flips_attempted: u64,
    /// Number of edge flips accepted
    pub edge_flips_accepted: u64,
}

impl MoveStatistics {
    /// Creates a new statistics tracker.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Records an attempted move.
    pub const fn record_attempt(&mut self, move_type: MoveType) {
        match move_type {
            MoveType::Move22 => self.moves_22_attempted += 1,
            MoveType::Move13Add => self.moves_13_attempted += 1,
            MoveType::Move31Remove => self.moves_31_attempted += 1,
            MoveType::EdgeFlip => self.edge_flips_attempted += 1,
        }
    }

    /// Records a successful move.
    pub const fn record_success(&mut self, move_type: MoveType) {
        match move_type {
            MoveType::Move22 => self.moves_22_accepted += 1,
            MoveType::Move13Add => self.moves_13_accepted += 1,
            MoveType::Move31Remove => self.moves_31_accepted += 1,
            MoveType::EdgeFlip => self.edge_flips_accepted += 1,
        }
    }

    /// Calculates acceptance rate for a specific move type.
    #[must_use]
    pub fn acceptance_rate(&self, move_type: MoveType) -> f64 {
        let (attempted, accepted) = match move_type {
            MoveType::Move22 => (self.moves_22_attempted, self.moves_22_accepted),
            MoveType::Move13Add => (self.moves_13_attempted, self.moves_13_accepted),
            MoveType::Move31Remove => (self.moves_31_attempted, self.moves_31_accepted),
            MoveType::EdgeFlip => (self.edge_flips_attempted, self.edge_flips_accepted),
        };

        if attempted == 0 {
            0.0
        } else {
            f64::from(u32::try_from(accepted).unwrap_or_default())
                / f64::from(u32::try_from(attempted).unwrap_or_default())
        }
    }

    /// Calculates overall acceptance rate.
    #[must_use]
    pub fn total_acceptance_rate(&self) -> f64 {
        let total_attempted = self.moves_22_attempted
            + self.moves_13_attempted
            + self.moves_31_attempted
            + self.edge_flips_attempted;
        let total_accepted = self.moves_22_accepted
            + self.moves_13_accepted
            + self.moves_31_accepted
            + self.edge_flips_accepted;

        if total_attempted == 0 {
            0.0
        } else {
            f64::from(u32::try_from(total_accepted).unwrap_or_default())
                / f64::from(u32::try_from(total_attempted).unwrap_or_default())
        }
    }
}

/// Ergodic move system for CDT triangulations.
pub struct ErgodicsSystem {
    /// Move statistics
    pub stats: MoveStatistics,
    /// Random number generator
    rng: rand::rngs::ThreadRng,
}

impl ErgodicsSystem {
    /// Creates a new ergodics system.
    #[must_use]
    pub fn new() -> Self {
        Self {
            stats: MoveStatistics::new(),
            rng: rand::rng(),
        }
    }

    /// Selects a random move type.
    #[must_use]
    pub fn select_random_move(&mut self) -> MoveType {
        match self.rng.random_range(0..4) {
            0 => MoveType::Move22,
            1 => MoveType::Move13Add,
            2 => MoveType::Move31Remove,
            _ => MoveType::EdgeFlip,
        }
    }

    /// Attempts a (2,2) move on the triangulation.
    ///
    /// A (2,2) move flips an edge between two triangles, changing the
    /// local triangulation while preserving the number of vertices.
    pub fn attempt_22_move(&mut self, _triangulation: &mut Vec<Vec<usize>>) -> MoveResult {
        self.stats.record_attempt(MoveType::Move22);

        // TODO: Implement actual (2,2) move logic with delaunay integration
        // For now, randomly accept/reject to test the framework
        if generate_random_float() > 0.5 {
            self.stats.record_success(MoveType::Move22);
            MoveResult::Success
        } else {
            MoveResult::CausalityViolation
        }
    }

    /// Attempts a (1,3) move on the triangulation.
    ///
    /// A (1,3) move adds a vertex by subdividing an existing triangle
    /// into three smaller triangles.
    pub fn attempt_13_move(&mut self, _triangulation: &mut Vec<Vec<usize>>) -> MoveResult {
        self.stats.record_attempt(MoveType::Move13Add);

        // TODO: Implement actual (1,3) move logic with delaunay integration
        if generate_random_float() > 0.3 {
            self.stats.record_success(MoveType::Move13Add);
            MoveResult::Success
        } else {
            MoveResult::GeometricViolation
        }
    }

    /// Attempts a (3,1) move on the triangulation.
    ///
    /// A (3,1) move removes a vertex by merging its surrounding triangles
    /// into a single triangle.
    pub fn attempt_31_move(&mut self, _triangulation: &mut Vec<Vec<usize>>) -> MoveResult {
        self.stats.record_attempt(MoveType::Move31Remove);

        // TODO: Implement actual (3,1) move logic with delaunay integration
        if generate_random_float() > 0.4 {
            self.stats.record_success(MoveType::Move31Remove);
            MoveResult::Success
        } else {
            MoveResult::CausalityViolation
        }
    }

    /// Attempts an edge flip move on the triangulation.
    ///
    /// An edge flip maintains the Delaunay property while potentially
    /// changing the causal structure.
    pub fn attempt_edge_flip(&mut self, _triangulation: &mut Vec<Vec<usize>>) -> MoveResult {
        self.stats.record_attempt(MoveType::EdgeFlip);

        // TODO: Implement actual edge flip logic with delaunay integration
        if generate_random_float() > 0.2 {
            self.stats.record_success(MoveType::EdgeFlip);
            MoveResult::Success
        } else {
            MoveResult::CausalityViolation
        }
    }

    /// Attempts a random ergodic move on the triangulation.
    pub fn attempt_random_move(&mut self, triangulation: &mut Vec<Vec<usize>>) -> MoveResult {
        let move_type = self.select_random_move();
        match move_type {
            MoveType::Move22 => self.attempt_22_move(triangulation),
            MoveType::Move13Add => self.attempt_13_move(triangulation),
            MoveType::Move31Remove => self.attempt_31_move(triangulation),
            MoveType::EdgeFlip => self.attempt_edge_flip(triangulation),
        }
    }
}

impl Default for ErgodicsSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_move_statistics() {
        let mut stats = MoveStatistics::new();

        // Test recording attempts and successes
        stats.record_attempt(MoveType::Move22);
        stats.record_attempt(MoveType::Move22);
        stats.record_success(MoveType::Move22);

        assert_eq!(stats.moves_22_attempted, 2);
        assert_eq!(stats.moves_22_accepted, 1);
        assert_relative_eq!(stats.acceptance_rate(MoveType::Move22), 0.5);
    }

    #[test]
    fn test_ergodics_system() {
        let mut system = ErgodicsSystem::new();
        let mut triangulation = vec![vec![0, 1, 2]];

        // Test that moves can be attempted
        let result = system.attempt_22_move(&mut triangulation);
        assert!(matches!(
            result,
            MoveResult::Success | MoveResult::CausalityViolation
        ));

        // Check that statistics are updated
        assert_eq!(system.stats.moves_22_attempted, 1);
    }

    #[test]
    fn test_random_move_selection() {
        let mut system = ErgodicsSystem::new();

        // Test that we get different move types
        let mut move_types = std::collections::HashSet::new();
        for _ in 0..100 {
            move_types.insert(system.select_random_move());
        }

        // Should get multiple different move types
        assert!(move_types.len() > 1);
    }

    #[test]
    fn test_total_acceptance_rate() {
        let mut stats = MoveStatistics::new();

        // Add some test data
        stats.record_attempt(MoveType::Move22);
        stats.record_success(MoveType::Move22);
        stats.record_attempt(MoveType::Move13Add);

        assert_relative_eq!(stats.total_acceptance_rate(), 0.5);
    }
}
