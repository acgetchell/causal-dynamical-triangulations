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
}
