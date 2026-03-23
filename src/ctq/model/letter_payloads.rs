use serde::Deserialize;
use validator::Validate;

#[derive(Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct LetterCheckPayload {
    #[validate(custom(function = "is_lowercase_alphabetic"))]
    pub letter_to_check: char,

    #[validate(custom(function = "is_lowercase_alphabetic"))]
    pub cipher_letter: char,
}

#[derive(Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct LetterSolvePayload {
    #[validate(custom(function = "is_lowercase_alphabetic"))]
    pub cipher_letter: char,
}

fn is_lowercase_alphabetic(c: &char) -> Result<(), validator::ValidationError> {
    if c.is_ascii_lowercase() && c.is_ascii_alphabetic() {
        Ok(())
    } else {
        Err(validator::ValidationError::new("must be lower case letter"))
    }
}
