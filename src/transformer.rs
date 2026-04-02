use serde_json::Value;
use std::collections::HashMap;

use crate::validators::validate_cipher_map;

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
