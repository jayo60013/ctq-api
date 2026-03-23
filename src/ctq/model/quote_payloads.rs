use std::collections::{HashMap, HashSet};

use serde::Deserialize;
use validator::Validate;

#[derive(Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct QuoteCheckPayload {
    #[validate(custom(function = "validate_cipher_map"))]
    pub cipher_map: HashMap<char, char>,
}

fn validate_cipher_map(map: &HashMap<char, char>) -> Result<(), validator::ValidationError> {
    if map.len() > 26 {
        return Err(validator::ValidationError::new("Cipher map too big"));
    }

    let is_keys_and_values_valid = map.iter().all(|(&k, &v)| {
        k.is_ascii_lowercase()
            && k.is_ascii_alphabetic()
            && v.is_ascii_lowercase()
            && v.is_ascii_alphabetic()
    });

    if !is_keys_and_values_valid {
        return Err(validator::ValidationError::new(
            "Cipher map contains invalid characters",
        ));
    }

    let is_contains_duplicate = map.values().collect::<HashSet<_>>().len() != map.len();
    if is_contains_duplicate {
        return Err(validator::ValidationError::new(
            "Cipher map contains duplicate mappings",
        ));
    }

    Ok(())
}
