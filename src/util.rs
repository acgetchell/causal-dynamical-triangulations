use rand::random;

/// Generates a random floating-point number between 0.0 and 1.0.
///
/// # Returns
///
/// A random `f64` value in the range [0.0, 1.0).
#[must_use]
pub fn generate_random_float() -> f64 {
    random::<f64>()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_generate_random_float() {
        let result = generate_random_float();
        assert!(result > 0.0);
        assert!(result < 1.0);
    }
}
