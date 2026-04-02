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

    #[test]
    fn test_check_letter_correct() {
        // Given
        let cipher_map = construct_cipher_map();

        // When
        let result = PuzzleService::check_letter('a', 'x', &cipher_map);

        // Then
        assert!(result);
    }

    #[test]
    fn test_check_letter_incorrect() {
        // Given
        let cipher_map = construct_cipher_map();

        // When
        let result = PuzzleService::check_letter('z', 'b', &cipher_map);

        // Then
        assert!(!result);
    }

    #[test]
    fn test_check_letter_not_found() {
        // Given
        let cipher_map = construct_cipher_map();

        // When
        let result = PuzzleService::check_letter('a', 'z', &cipher_map);

        // Then
        assert!(!result);
    }

    #[test]
    fn test_solve_letter_success() {
        // Given
        let cipher_map = construct_cipher_map();
        let expected_letter: char = 'x';

        // When
        let actual_letter = PuzzleService::solve_letter('a', &cipher_map).unwrap();

        // Then
        assert_eq!(actual_letter, expected_letter);
    }

    #[test]
    fn test_solve_letter_not_found() {
        // Given
        let cipher_map = construct_cipher_map();

        // When
        let result = PuzzleService::solve_letter('x', &cipher_map);

        // Then
        assert!(result.is_err());
    }

    #[test]
    fn test_check_quote_matches() {
        // Given
        let cipher_map = construct_cipher_map();

        // When
        let result = PuzzleService::check_quote(&cipher_map, &cipher_map);

        // Then
        assert!(result);
    }

    #[test]
    fn test_check_quote_different() {
        // Given
        let cipher_map = construct_cipher_map();
        let mut different_map = cipher_map.clone();
        different_map.insert('x', 'z');

        // When
        let result = PuzzleService::check_quote(&different_map, &cipher_map);

        // Then
        assert!(!result);
    }

    fn construct_cipher_map() -> HashMap<char, char> {
        HashMap::from([('a', 'x'), ('b', 'y'), ('c', 'z')])
    }
}
