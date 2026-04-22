use serde_json::Value;
use std::collections::HashMap;

use crate::models::ScoreDistributionBucket;
use crate::validators::validate_cipher_map;

/// Transforms raw score distribution data into buckets with percentages.
///
/// # Arguments
/// * `distribution_data` - Raw tuples of (score, score, count)
/// * `total_solved` - Total number of puzzles solved (for percentage calculation)
///
/// # Returns
/// Vector of `ScoreDistributionBucket` with score values and percentages calculated (as 0-100 scale)
pub fn build_score_distribution(
    distribution_data: &[(i32, i32, i64)],
    total_solved: i64,
) -> Vec<ScoreDistributionBucket> {
    build_score_distribution_with_rounding(
        distribution_data,
        total_solved,
        RoundingStrategy::NoRounding,
    )
}

/// Rounding strategy for percentage values
#[derive(Debug, Clone, Copy)]
pub enum RoundingStrategy {
    /// No rounding, return raw decimal (e.g., 0.333)
    NoRounding,
    /// Round to 1 decimal place percentage (e.g., 33.3)
    OneDecimalPercentage,
}

/// Transforms raw score distribution data into buckets with customizable rounding.
///
/// # Arguments
/// * `distribution_data` - Raw tuples of (score, score, count)
/// * `total_solved` - Total number of puzzles solved (for percentage calculation)
/// * `rounding` - How to round the percentage values
///
/// # Returns
/// Vector of `ScoreDistributionBucket` with score values and percentages calculated and rounded
pub fn build_score_distribution_with_rounding(
    distribution_data: &[(i32, i32, i64)],
    total_solved: i64,
    rounding: RoundingStrategy,
) -> Vec<ScoreDistributionBucket> {
    distribution_data
        .iter()
        .map(|(min, _max, count)| {
            #[allow(clippy::cast_precision_loss)]
            let percentage = if total_solved > 0 {
                let raw_percentage = (*count as f64 / total_solved as f64) * 100.0;
                match rounding {
                    RoundingStrategy::NoRounding => raw_percentage,
                    RoundingStrategy::OneDecimalPercentage => {
                        // Round to 1 decimal place
                        (raw_percentage * 10.0).round() / 10.0
                    }
                }
            } else {
                0.0
            };

            ScoreDistributionBucket {
                score: *min,
                count: *count,
                percentage,
            }
        })
        .collect()
}

pub fn parse_cipher_map_from_json(value: &Value) -> Result<HashMap<char, char>, String> {
    match value.as_object() {
        Some(obj) => {
            let mut map = HashMap::new();
            for (k, v) in obj {
                if k.len() != 1 {
                    return Err("cipher_map keys must be single characters".to_string());
                }
                let key_char = k.chars().next().unwrap();
                match v.as_str() {
                    Some(s) if s.len() == 1 => {
                        map.insert(key_char, s.chars().next().unwrap());
                    }
                    _ => {
                        return Err(
                            "cipher_map values must be single character strings".to_string()
                        );
                    }
                }
            }
            validate_cipher_map(&map)?;
            Ok(map)
        }
        None => Err("cipher_map must be a JSON object".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_score_distribution() {
        // Given - Individual scores 0, 1, 2 with counts
        let distribution_data = vec![(0, 0, 10i64), (1, 1, 20i64), (2, 2, 30i64)];
        let total_solved = 60i64;

        // When
        let result = build_score_distribution(&distribution_data, total_solved);

        // Then
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].score, 0);
        assert_eq!(result[0].count, 10);
        assert!((result[0].percentage - (10.0 / 60.0 * 100.0)).abs() < 0.0001);
        assert_eq!(result[1].score, 1);
        assert_eq!(result[1].count, 20);
        assert!((result[1].percentage - (20.0 / 60.0 * 100.0)).abs() < 0.0001);
        assert_eq!(result[2].score, 2);
        assert_eq!(result[2].count, 30);
        assert!((result[2].percentage - (30.0 / 60.0 * 100.0)).abs() < 0.0001);
    }

    #[test]
    fn test_build_score_distribution_zero_solved() {
        // When
        let result = build_score_distribution(&[(0, 0, 10i64)], 0i64);

        // Then
        assert_eq!(result[0].score, 0);
        assert!((result[0].percentage - 0.0).abs() < 0.0001);
    }

    #[test]
    fn test_build_score_distribution_empty() {
        // When
        let result = build_score_distribution(&[], 100i64);

        // Then
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_build_score_distribution_with_rounding_no_rounding() {
        // Given - 1 out of 3 = 33.333...%
        let distribution_data = vec![(0, 0, 1i64), (1, 1, 1i64), (2, 2, 1i64)];
        let total_solved = 3i64;

        // When
        let result = build_score_distribution_with_rounding(
            &distribution_data,
            total_solved,
            RoundingStrategy::NoRounding,
        );

        // Then - Should be 33.333...
        assert_eq!(result[0].score, 0);
        assert!((result[0].percentage - (1.0 / 3.0 * 100.0)).abs() < 0.0001);
    }

    #[test]
    fn test_build_score_distribution_with_rounding_one_decimal_percentage() {
        // Given - 1 out of 3 = 33.3%
        let distribution_data = vec![(0, 0, 1i64), (1, 1, 1i64), (2, 2, 1i64)];
        let total_solved = 3i64;

        // When
        let result = build_score_distribution_with_rounding(
            &distribution_data,
            total_solved,
            RoundingStrategy::OneDecimalPercentage,
        );

        // Then - Should be 33.3 (rounded to 1 decimal)
        assert_eq!(result[0].score, 0);
        assert!((result[0].percentage - 33.3).abs() < 0.01);
    }

    #[test]
    fn test_parse_cipher_map_from_json_valid() {
        // Given
        let json = serde_json::json!({ "a": "b", "c": "d" });
        let expected = HashMap::from([('a', 'b'), ('c', 'd')]);

        // When
        let actual = parse_cipher_map_from_json(&json).unwrap();

        // Then
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_parse_cipher_map_from_json_invalid_not_object() {
        // Given
        let json = serde_json::json!([1, 2, 3]);

        // When
        let result = parse_cipher_map_from_json(&json);

        // Then
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_cipher_map_from_json_invalid_key() {
        // Given
        let json = serde_json::json!({ "ab": "c" });

        // When
        let result = parse_cipher_map_from_json(&json);

        // Then
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_cipher_map_from_json_invalid_value() {
        // Given
        let json = serde_json::json!({ "a": "bc" });

        // When
        let result = parse_cipher_map_from_json(&json);

        // Then
        assert!(result.is_err());
    }
}
