use std::collections::HashMap;

#[derive(Debug)]
pub struct DailyPuzzle {
    pub cipher_quote: String,
    pub author: String,
    pub date_string: String,
    pub day_number: u16,
    pub cipher_map: HashMap<char, char>,
}

pub fn get_empty_daily_puzzle() -> DailyPuzzle {
    DailyPuzzle {
        cipher_quote: "".to_string(),
        author: "".to_string(),
        date_string: "".to_string(),
        day_number: 0,
        cipher_map: HashMap::new(),
    }
}
