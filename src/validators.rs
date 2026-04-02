use std::collections::HashMap;

#[allow(clippy::trivially_copy_pass_by_ref)]
pub fn validate_lowercase_letter(c: &char) -> Result<(), validator::ValidationError> {
    if c.is_ascii_lowercase() && c.is_ascii_alphabetic() {
        Ok(())
    } else {
        Err(validator::ValidationError::new("must_be_lowercase_letter"))
    }
}

pub fn validate_cipher_map(cipher_map: &HashMap<char, char>) -> Result<(), String> {
    if cipher_map.is_empty() {
        return Err("cipher_map cannot be empty".to_string());
    }

    for (k, v) in cipher_map {
        if !k.is_ascii_alphabetic() || !v.is_ascii_alphabetic() {
            return Err("cipher_map keys and values must be alphabetic characters".to_string());
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_lowercase_letter_valid() {
        assert!(validate_lowercase_letter(&'a').is_ok());
        assert!(validate_lowercase_letter(&'z').is_ok());
    }

    #[test]
    fn test_validate_lowercase_letter_invalid() {
        assert!(validate_lowercase_letter(&'A').is_err());
        assert!(validate_lowercase_letter(&'1').is_err());
    }

    #[test]
    fn test_validate_cipher_map_valid() {
        let map = HashMap::from([('a', 'b'), ('c', 'd')]);
        assert!(validate_cipher_map(&map).is_ok());
    }

    #[test]
    fn test_validate_cipher_map_empty() {
        let map: HashMap<char, char> = HashMap::new();
        assert!(validate_cipher_map(&map).is_err());
    }
}
