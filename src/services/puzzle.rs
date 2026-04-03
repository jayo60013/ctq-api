use std::collections::HashMap;

use crate::error::ApiError;

pub struct PuzzleService;

impl PuzzleService {
    pub fn check_letter(
        cipher_letter: char,
        letter_to_check: char,
        cipher_map: &HashMap<char, char>,
    ) -> bool {
        cipher_map
            .get(&cipher_letter)
            .is_some_and(|&correct_letter| correct_letter == letter_to_check)
    }

    pub fn solve_letter(
        cipher_letter: char,
        cipher_map: &HashMap<char, char>,
    ) -> Result<char, ApiError> {
        cipher_map
            .get(&cipher_letter)
            .copied()
            .ok_or_else(|| ApiError::ValidationError("Letter not found in puzzle".to_string()))
    }

    pub fn check_quote(
        proposed_map: &HashMap<char, char>,
        actual_map: &HashMap<char, char>,
    ) -> bool {
        proposed_map == actual_map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn construct_cipher_map() -> HashMap<char, char> {
        let mut map = HashMap::new();
        map.insert('a', 'x');
        map.insert('z', 'b');
        map
    }

    #[test]
    fn test_check_letter_correct() {
        let cipher_map = construct_cipher_map();
        let result = PuzzleService::check_letter('a', 'x', &cipher_map);
        assert!(result);
    }

    #[test]
    fn test_check_letter_incorrect() {
        let cipher_map = construct_cipher_map();
        let result = PuzzleService::check_letter('z', 'x', &cipher_map);
        assert!(!result);
    }

    #[test]
    fn test_solve_letter_success() {
        let cipher_map = construct_cipher_map();
        let result = PuzzleService::solve_letter('a', &cipher_map).unwrap();
        assert_eq!(result, 'x');
    }

    #[test]
    fn test_solve_letter_not_found() {
        let cipher_map = construct_cipher_map();
        let result = PuzzleService::solve_letter('x', &cipher_map);
        assert!(result.is_err());
    }
}
