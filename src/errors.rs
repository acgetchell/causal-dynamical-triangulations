//! Error types for the CDT library.

use std::fmt;

/// Main error type for CDT operations.
#[derive(Debug, Clone, PartialEq)]
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
    /// Delaunay triangulation generation failed with detailed context
    DelaunayGenerationFailed {
        /// Number of vertices requested for the triangulation
        vertex_count: u32,
        /// Coordinate range used for generation
        coordinate_range: (f64, f64),
        /// Attempt number when the failure occurred
        attempt: u32,
        /// Description of the underlying error that caused the failure
        underlying_error: String,
    },
    /// Invalid generation parameters detected before attempting triangulation
    InvalidGenerationParameters {
        /// Description of the specific parameter issue
        issue: String,
        /// The actual value that was provided
        provided_value: String,
        /// The expected range or constraint for the parameter
        expected_range: String,
    },
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
            Self::DelaunayGenerationFailed {
                vertex_count,
                coordinate_range,
                attempt,
                underlying_error,
            } => write!(
                f,
                "Delaunay triangulation generation failed: {vertex_count} vertices, range [{}, {}], attempt {attempt}: {underlying_error}",
                coordinate_range.0, coordinate_range.1
            ),
            Self::InvalidGenerationParameters {
                issue,
                provided_value,
                expected_range,
            } => write!(
                f,
                "Invalid triangulation parameters: {issue} (got: {provided_value}, expected: {expected_range})",
            ),
        }
    }
}

impl std::error::Error for CdtError {}

/// Result type for CDT operations.
pub type CdtResult<T> = Result<T, CdtError>;
