use std::collections::{HashMap, HashSet};

use chrono::{Local, NaiveDate};
use log::info;
use rand::seq::SliceRandom;
use sqlx::PgPool;

use crate::{
    models::{daily_puzzle::DailyPuzzle, quote::Quote},
    utils::date_time_utils::format_date_string,
};

pub async fn get_daily_puzzle_entity(pool: &PgPool) -> anyhow::Result<DailyPuzzle> {
    let daily_quote = get_daily_quote(pool).await?;
    let cipher_map = get_cipher_map(&daily_quote.quote);

    let cipher_quote = encode(&daily_quote.quote, &cipher_map);

    dotenvy::dotenv().ok();
    let start_date = NaiveDate::parse_from_str(&std::env::var("START_DATE")?, "%Y-%m-%d")
        .expect("Invalid start_date format");
    let today = Local::now().date_naive();

    let day_number = (today - start_date).num_days() + 1;
    let date_string = format_date_string(today);

    let daily_puzzle = DailyPuzzle {
        cipher_quote,
        author: daily_quote.author,
        source: daily_quote.source,
        date_string,
        day_number: day_number as u16,
        cipher_map: inverse(cipher_map),
    };

    Ok(daily_puzzle)
}

async fn get_daily_quote(pool: &PgPool) -> anyhow::Result<Quote> {
    dotenvy::dotenv().ok();
    let start_date = NaiveDate::parse_from_str(&std::env::var("START_DATE")?, "%Y-%m-%d")
        .expect("Invalid start_date format");
    let today = Local::now().date_naive();

    info!("Fetching count from quotes table");
    let quotes_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM quotes")
        .fetch_one(pool)
        .await?;

    // Id is the difference in from start_date & today + 1
    let days = (today - start_date).num_days();
    let id = ((days + 1).rem_euclid(quotes_count)) as i64;
    info!("id={}", id);

    let quote =
        sqlx::query_as::<_, Quote>("SELECT quote, author, source FROM quotes WHERE id=$1")
        .bind(id)
        .fetch_one(pool)
        .await?;

    Ok(quote)
}

fn get_cipher_map(quote: &str) -> HashMap<char, char> {
    let quote_char_set: HashSet<char> = quote.to_lowercase().chars().collect();
    let mut rng = rand::rng();

    let alphabet: Vec<char> = ('a'..='z').collect();

    std::iter::repeat_with(|| {
        let mut shuffled = alphabet.clone();
        shuffled.shuffle(&mut rng);
        shuffled
    })
    .find(|shuffled| alphabet.iter().zip(shuffled).all(|(a, b)| a != b))
    .map(|shuffled| {
        alphabet
            .into_iter()
            .zip(shuffled)
            .filter(|(k, _)| quote_char_set.contains(k))
            .collect()
    })
    .unwrap_or_default()
}

fn inverse(map: HashMap<char, char>) -> HashMap<char, char> {
    map.iter().map(|(k, v)| (*v, *k)).collect()
}

fn encode(quote: &str, cipher_map: &HashMap<char, char>) -> String {
    quote
        .to_lowercase()
        .chars()
        .map(|c| *cipher_map.get(&c).unwrap_or(&c))
        .collect()
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn test_cipher_map_uses_each_letter_once() {
        // Given
        let quote = "Every flower is a soul blossoming in nature.";

        // When
        let actual_cipher_map = get_cipher_map(quote);

        // Then
        let keys: HashSet<char> = actual_cipher_map.keys().copied().collect();
        let values: HashSet<char> = actual_cipher_map.values().copied().collect();

        assert_eq!(keys.len(), 17);
        assert_eq!(values.len(), 17);
    }

    #[test]
    fn test_cipher_map_does_not_map_same_letter() {
        // Given
        let quote = "Every flower is a soul blossoming in nature.";

        // When
        let actual_cipher_map = get_cipher_map(quote);

        // Then
        let is_same_map = actual_cipher_map.iter().any(|(k, v)| k == v);
        assert_eq!(is_same_map, false);
    }

    #[test]
    fn test_encode() {
        // Given
        let quote = "Every flower is a soul blossoming in nature.";
        let expected_encoded_quote = "tutsr klayts gf d faml plaffavgwh gw wdnmst.";

        let cipher_map = HashMap::from([
            ('o', 'a'),
            ('a', 'd'),
            ('s', 'f'),
            ('i', 'g'),
            ('g', 'h'),
            ('f', 'k'),
            ('l', 'l'),
            ('u', 'm'),
            ('t', 'n'),
            ('b', 'p'),
            ('y', 'r'),
            ('r', 's'),
            ('e', 't'),
            ('v', 'u'),
            ('m', 'v'),
            ('n', 'w'),
            ('w', 'y'),
        ]);

        // When
        let actual_encoded_quote = encode(quote, &cipher_map);

        // Then
        assert_eq!(actual_encoded_quote, expected_encoded_quote);
    }

    #[test]
    fn test_inverse() {
        // Given
        let cipher_map = HashMap::from([('a', 'b'), ('y', 'z')]);
        let expected_inversed_cipher_map = HashMap::from([('b', 'a'), ('z', 'y')]);

        // When
        let actual_inversed_cipher_map = inverse(cipher_map);

        // Then
        assert_eq!(actual_inversed_cipher_map, expected_inversed_cipher_map);
    }
}
