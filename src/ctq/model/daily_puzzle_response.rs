use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DailyPuzzleResponse {
    pub author: String,
    pub cipher_quote: String,
    pub date_string: String,
    pub day_number: u16,
}

pub fn get_empty_daily_puzzle_response() -> DailyPuzzleResponse {
    DailyPuzzleResponse {
        author: "".to_string(),
        cipher_quote: "".to_string(),
        date_string: "".to_string(),
        day_number: 0,
    }
}
