//! Configuration management for CDT simulations.
//!
//! This module provides structured configuration for various aspects of
//! Causal Dynamical Triangulation simulations, including:
//! - Simulation parameters (temperature, steps, etc.)
//! - Action calculation parameters (coupling constants, cosmological constant)
//! - Triangulation generation parameters
//! - Runtime behavior options

use crate::cdt::action::ActionConfig;
use crate::cdt::metropolis::MetropolisConfig;
use clap::Parser;
use std::env;
use std::path::{Component, Path, PathBuf};

/// Main configuration structure for CDT simulations.
///
/// This combines all configuration options for the CDT simulation,
/// including triangulation generation, action calculation, and
/// Metropolis algorithm parameters.
#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct CdtConfig {
    /// Dimensionality of the triangulation
    #[arg(short, long, value_parser = clap::value_parser!(u8).range(2..4))]
    pub dimension: Option<u8>,

    /// Number of vertices in the initial triangulation
    #[arg(short, long, value_parser = clap::value_parser!(u32).range(3..))]
    pub vertices: u32,

    /// Number of timeslices in the triangulation
    #[arg(short, long, value_parser = clap::value_parser!(u32).range(1..))]
    pub timeslices: u32,

    /// Temperature for Metropolis algorithm
    #[arg(long, default_value = "1.0")]
    pub temperature: f64,

    /// Number of Monte Carlo steps to execute
    #[arg(long, default_value = "1000")]
    pub steps: u32,

    /// Number of thermalization steps (before measurements begin)
    #[arg(long, default_value = "100")]
    pub thermalization_steps: u32,

    /// Measurement frequency (take measurement every N steps)
    #[arg(long, default_value = "10", value_parser = clap::value_parser!(u32).range(1..))]
    pub measurement_frequency: u32,

    /// Coupling constant κ₀ for vertices in the action
    #[arg(long, default_value = "1.0")]
    pub coupling_0: f64,

    /// Coupling constant κ₂ for triangles in the action
    #[arg(long, default_value = "1.0")]
    pub coupling_2: f64,

    /// Cosmological constant λ in the action
    #[arg(long, default_value = "0.1")]
    pub cosmological_constant: f64,

    /// Run full CDT simulation (default: false, just generate triangulation)
    #[arg(long, default_value = "false")]
    pub simulate: bool,
}

/// Controls how dimension overrides are applied when merging configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DimensionOverride {
    /// Replace the dimension with the supplied value.
    Value(u8),
    /// Clear the dimension so it falls back to the default.
    Clear,
}

/// A collection of optional override values for [`CdtConfig`].
///
/// Each field is optional, allowing callers to override only the configuration entries
/// that need changing while leaving the rest untouched.
#[derive(Debug, Default, Clone, Copy)]
pub struct CdtConfigOverrides {
    /// Optional override for the triangulation dimension.
    pub dimension: Option<DimensionOverride>,
    /// Optional override for the vertex count.
    pub vertices: Option<u32>,
    /// Optional override for the timeslice count.
    pub timeslices: Option<u32>,
    /// Optional override for the temperature.
    pub temperature: Option<f64>,
    /// Optional override for the total number of steps.
    pub steps: Option<u32>,
    /// Optional override for the number of thermalization steps.
    pub thermalization_steps: Option<u32>,
    /// Optional override for the measurement frequency.
    pub measurement_frequency: Option<u32>,
    /// Optional override for κ₀.
    pub coupling_0: Option<f64>,
    /// Optional override for κ₂.
    pub coupling_2: Option<f64>,
    /// Optional override for the cosmological constant λ.
    pub cosmological_constant: Option<f64>,
    /// Optional override for the simulation flag.
    pub simulate: Option<bool>,
}

impl CdtConfig {
    /// Merges this configuration with a set of override values, returning a new configuration.
    ///
    /// Override fields that are `None` are ignored, leaving the original configuration values
    /// unchanged. When an override value is provided, it replaces the corresponding field in
    /// the returned configuration.
    #[must_use]
    pub fn merge_with_override(&self, overrides: &CdtConfigOverrides) -> Self {
        let mut merged = self.clone();

        if let Some(dimension_override) = overrides.dimension {
            match dimension_override {
                DimensionOverride::Value(value) => {
                    merged.dimension = Some(value);
                }
                DimensionOverride::Clear => {
                    merged.dimension = None;
                }
            }
        }

        if let Some(vertices) = overrides.vertices {
            merged.vertices = vertices;
        }

        if let Some(timeslices) = overrides.timeslices {
            merged.timeslices = timeslices;
        }

        if let Some(temperature) = overrides.temperature {
            merged.temperature = temperature;
        }

        if let Some(steps) = overrides.steps {
            merged.steps = steps;
        }

        if let Some(thermalization_steps) = overrides.thermalization_steps {
            merged.thermalization_steps = thermalization_steps;
        }

        if let Some(measurement_frequency) = overrides.measurement_frequency {
            merged.measurement_frequency = measurement_frequency;
        }

        if let Some(coupling_0) = overrides.coupling_0 {
            merged.coupling_0 = coupling_0;
        }

        if let Some(coupling_2) = overrides.coupling_2 {
            merged.coupling_2 = coupling_2;
        }

        if let Some(cosmological_constant) = overrides.cosmological_constant {
            merged.cosmological_constant = cosmological_constant;
        }

        if let Some(simulate) = overrides.simulate {
            merged.simulate = simulate;
        }

        merged
    }

    /// Resolves a candidate path against a base directory, expanding user home references
    /// and normalizing relative segments (e.g., `.` and `..`).
    #[must_use]
    pub fn resolve_path(base_dir: impl AsRef<Path>, candidate: impl AsRef<Path>) -> PathBuf {
        let candidate = candidate.as_ref();

        if candidate.is_absolute() {
            return normalize_components(candidate);
        }

        if let Some(candidate_str) = candidate.to_str() {
            if let Some(stripped) = candidate_str.strip_prefix("~/") {
                if let Ok(home) = env::var("HOME") {
                    let path = PathBuf::from(home).join(stripped);
                    return normalize_components(&path);
                }
            } else if candidate_str == "~"
                && let Ok(home) = env::var("HOME")
            {
                let path = PathBuf::from(home);
                return normalize_components(&path);
            }
        }

        let joined = base_dir.as_ref().join(candidate);
        normalize_components(&joined)
    }
}

fn normalize_components(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();

    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                if normalized.as_os_str().is_empty() {
                    continue;
                }
                normalized.pop();
            }
            Component::RootDir | Component::Prefix(_) => {
                normalized.push(component.as_os_str());
            }
            Component::Normal(segment) => {
                normalized.push(segment);
            }
        }
    }

    if normalized.as_os_str().is_empty() {
        PathBuf::from(Component::CurDir.as_os_str())
    } else {
        normalized
    }
}

impl CdtConfig {
    /// Builds a new instance of `CdtConfig` from command line arguments.
    #[must_use]
    pub fn from_args() -> Self {
        Self::parse()
    }

    /// Creates a new `CdtConfig` with specified basic parameters and default action parameters.
    #[must_use]
    pub const fn new(vertices: u32, timeslices: u32) -> Self {
        Self {
            dimension: Some(2),
            vertices,
            timeslices,
            temperature: 1.0,
            steps: 1000,
            thermalization_steps: 100,
            measurement_frequency: 10,
            coupling_0: 1.0,
            coupling_2: 1.0,
            cosmological_constant: 0.1,
            simulate: true,
        }
    }

    /// Creates a `MetropolisConfig` from this configuration.
    #[must_use]
    pub const fn to_metropolis_config(&self) -> MetropolisConfig {
        MetropolisConfig::new(
            self.temperature,
            self.steps,
            self.thermalization_steps,
            self.measurement_frequency,
        )
    }

    /// Creates an `ActionConfig` from this configuration.
    #[must_use]
    pub const fn to_action_config(&self) -> ActionConfig {
        ActionConfig::new(self.coupling_0, self.coupling_2, self.cosmological_constant)
    }

    /// Gets the effective dimension (defaults to 2 if not specified).
    #[must_use]
    pub const fn dimension(&self) -> u8 {
        match self.dimension {
            Some(d) => d,
            None => 2,
        }
    }

    /// Validates the configuration parameters.
    ///
    /// # Errors
    ///
    /// Returns an error message if any parameters are invalid.
    pub fn validate(&self) -> Result<(), String> {
        if self.vertices < 3 {
            return Err("Number of vertices must be at least 3".to_string());
        }

        if self.timeslices == 0 {
            return Err("Number of timeslices must be at least 1".to_string());
        }

        if let Some(dim) = self.dimension
            && !(2..=3).contains(&dim)
        {
            return Err(format!(
                "Unsupported dimension: {dim}. Only 2D and 3D are supported."
            ));
        }

        if self.temperature <= 0.0 {
            return Err("Temperature must be positive".to_string());
        }

        if self.steps == 0 {
            return Err("Number of steps must be positive".to_string());
        }

        if self.measurement_frequency == 0 {
            return Err("Measurement frequency must be positive".to_string());
        }

        if self.measurement_frequency > self.steps {
            return Err("Measurement frequency cannot be greater than total steps".to_string());
        }

        Ok(())
    }
}

/// Configuration preset for quick testing.
#[derive(Debug, Clone)]
pub struct TestConfig;

impl TestConfig {
    /// Creates a small, fast configuration suitable for unit tests.
    #[must_use]
    pub const fn small() -> CdtConfig {
        CdtConfig {
            dimension: Some(2),
            vertices: 16,
            timeslices: 2,
            temperature: 1.0,
            steps: 10,
            thermalization_steps: 2,
            measurement_frequency: 2,
            coupling_0: 1.0,
            coupling_2: 1.0,
            cosmological_constant: 0.1,
            simulate: true,
        }
    }

    /// Creates a medium-sized configuration for integration tests.
    #[must_use]
    pub const fn medium() -> CdtConfig {
        CdtConfig {
            dimension: Some(2),
            vertices: 64,
            timeslices: 4,
            temperature: 1.0,
            steps: 100,
            thermalization_steps: 20,
            measurement_frequency: 5,
            coupling_0: 1.0,
            coupling_2: 1.0,
            cosmological_constant: 0.1,
            simulate: true,
        }
    }

    /// Creates a large configuration for performance testing.
    #[must_use]
    pub const fn large() -> CdtConfig {
        CdtConfig {
            dimension: Some(2),
            vertices: 256,
            timeslices: 8,
            temperature: 1.0,
            steps: 1000,
            thermalization_steps: 100,
            measurement_frequency: 10,
            coupling_0: 1.0,
            coupling_2: 1.0,
            cosmological_constant: 0.1,
            simulate: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    use std::env;
    use std::path::PathBuf;

    #[test]
    fn test_config_new() {
        let config = CdtConfig::new(32, 3);
        assert_eq!(config.vertices, 32);
        assert_eq!(config.timeslices, 3);
        assert_eq!(config.dimension(), 2);
        assert!(config.simulate);
    }

    #[test]
    fn test_config_conversions() {
        let config = CdtConfig::new(64, 4);

        let metropolis_config = config.to_metropolis_config();
        assert_relative_eq!(metropolis_config.temperature, 1.0);
        assert_eq!(metropolis_config.steps, 1000);

        let action_config = config.to_action_config();
        assert_relative_eq!(action_config.coupling_0, 1.0);
        assert_relative_eq!(action_config.coupling_2, 1.0);
        assert_relative_eq!(action_config.cosmological_constant, 0.1);
    }

    #[test]
    fn test_config_validation() {
        let valid_config = CdtConfig::new(32, 3);
        assert!(valid_config.validate().is_ok());

        let invalid_vertices = CdtConfig {
            vertices: 2,
            ..CdtConfig::new(32, 3)
        };
        assert!(invalid_vertices.validate().is_err());

        let invalid_timeslices = CdtConfig {
            timeslices: 0,
            ..CdtConfig::new(32, 3)
        };
        assert!(invalid_timeslices.validate().is_err());

        let invalid_temperature = CdtConfig {
            temperature: -1.0,
            ..CdtConfig::new(32, 3)
        };
        assert!(invalid_temperature.validate().is_err());

        let invalid_measurement_frequency = CdtConfig {
            measurement_frequency: 0,
            ..CdtConfig::new(32, 3)
        };
        assert!(invalid_measurement_frequency.validate().is_err());

        let invalid_steps = CdtConfig {
            steps: 0,
            ..CdtConfig::new(32, 3)
        };
        assert!(invalid_steps.validate().is_err());

        let invalid_dimension = CdtConfig {
            dimension: Some(4),
            ..CdtConfig::new(32, 3)
        };
        let error = invalid_dimension.validate().unwrap_err();
        assert!(
            error.contains("Unsupported dimension"),
            "unexpected validation error: {error}"
        );

        let measurement_frequency_exceeds_steps = CdtConfig {
            measurement_frequency: 2_000,
            ..CdtConfig::new(32, 3)
        };
        assert!(measurement_frequency_exceeds_steps.validate().is_err());
    }

    #[test]
    fn test_dimension_defaults_to_two_when_unspecified() {
        let config = CdtConfig {
            dimension: None,
            ..CdtConfig::new(32, 3)
        };
        assert_eq!(config.dimension(), 2);
    }

    #[test]
    fn test_preset_configs() {
        let small = TestConfig::small();
        assert!(small.validate().is_ok());
        assert_eq!(small.vertices, 16);
        assert_eq!(small.steps, 10);

        let medium = TestConfig::medium();
        assert!(medium.validate().is_ok());
        assert_eq!(medium.vertices, 64);
        assert_eq!(medium.steps, 100);

        let large = TestConfig::large();
        assert!(large.validate().is_ok());
        assert_eq!(large.vertices, 256);
        assert_eq!(large.steps, 1000);
    }

    #[test]
    fn test_merge_with_override_updates_specified_fields() {
        let base = CdtConfig::new(10, 2);
        let overrides = CdtConfigOverrides {
            dimension: Some(DimensionOverride::Value(3)),
            vertices: Some(42),
            temperature: Some(2.5),
            simulate: Some(false),
            ..CdtConfigOverrides::default()
        };

        let merged = base.merge_with_override(&overrides);

        assert_eq!(merged.dimension(), 3);
        assert_eq!(merged.vertices, 42);
        assert_relative_eq!(merged.temperature, 2.5);
        assert!(!merged.simulate);

        // Unspecified fields should remain unchanged.
        assert_eq!(merged.timeslices, base.timeslices);
        assert_eq!(merged.steps, base.steps);
    }

    #[test]
    fn test_merge_with_override_can_clear_dimension() {
        let base = CdtConfig::new(10, 2);
        let overrides = CdtConfigOverrides {
            dimension: Some(DimensionOverride::Clear),
            ..CdtConfigOverrides::default()
        };

        let merged = base.merge_with_override(&overrides);
        assert_eq!(merged.dimension, None);
        assert_eq!(merged.dimension(), 2); // dimension() defaults to 2 when None
    }

    #[test]
    fn test_resolve_path_with_absolute_path() {
        let abs = PathBuf::from("/tmp/example");
        let resolved = CdtConfig::resolve_path("/does/not/matter", &abs);
        assert_eq!(resolved, PathBuf::from("/tmp/example"));
    }

    #[test]
    fn test_resolve_path_with_relative_path() {
        let base = PathBuf::from("/tmp/base");
        let candidate = PathBuf::from("config/settings.toml");
        let resolved = CdtConfig::resolve_path(&base, &candidate);
        assert_eq!(resolved, PathBuf::from("/tmp/base/config/settings.toml"));
    }

    #[test]
    fn test_resolve_path_with_home_expansion() {
        let home = env::var("HOME").expect("HOME environment variable must be set for this test");
        let resolved = CdtConfig::resolve_path("/tmp", PathBuf::from("~/config.toml"));
        assert_eq!(resolved, PathBuf::from(home).join("config.toml"));
    }

    #[test]
    fn test_resolve_path_normalizes_navigation_components() {
        let base = PathBuf::from("/tmp/base");
        let candidate = PathBuf::from("configs/../settings.toml");
        let resolved = CdtConfig::resolve_path(&base, candidate);
        assert_eq!(resolved, PathBuf::from("/tmp/base/settings.toml"));
    }
}
