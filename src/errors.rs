//! Error types for the CDT library.

use std::fmt;

/// Main error type for CDT operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CdtError {
    /// Invalid triangulation parameters
    InvalidParameters(String),
    /// Triangulation generation failed
    TriangulationGeneration(String),
    /// Ergodic move failed
    ErgodicsFailure(String),
    /// Invalid dimension specified
    UnsupportedDimension(u32),
    /// Action calculation error
    ActionCalculation(String),
}

impl fmt::Display for CdtError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidParameters(msg) => write!(f, "Invalid parameters: {msg}"),
            Self::TriangulationGeneration(msg) => {
                write!(f, "Triangulation generation failed: {msg}")
            }
            Self::ErgodicsFailure(msg) => write!(f, "Ergodic move failed: {msg}"),
            Self::UnsupportedDimension(dim) => write!(
                f,
                "Unsupported dimension: {dim}. Only 2D is currently supported"
            ),
            Self::ActionCalculation(msg) => write!(f, "Action calculation error: {msg}"),
        }
    }
}

impl std::error::Error for CdtError {}

/// Result type for CDT operations.
pub type CdtResult<T> = Result<T, CdtError>;
