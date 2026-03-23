use std::fs::File;

use sqlx::PgPool;

use anyhow::{Context, Result};

use crate::{
    models::quote::Quote,
    utils::constants::{MAX_QUOTE_LENGTH, MIN_QUOTE_LENGTH, QUOTE_FILE_PATH},
};

pub async fn initialise_quotes_table(pool: &PgPool) -> Result<u64> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS quotes (
            id SERIAL PRIMARY KEY,
            author TEXT NOT NULL,
            quote TEXT NOT NULL
        )
        "#,
    )
    .execute(pool)
    .await
    .context("Failed to create quotes table")?;

    let quotes = get_all_quotes_from_json().context("Failed to read quotes file")?;

    let inserted = insert_quotes(pool, quotes)
        .await
        .context("Failed to insert quotes into table")?;

    Ok(inserted)
}

fn get_all_quotes_from_json() -> Result<Vec<Quote>> {
    let file = File::open(QUOTE_FILE_PATH)?;
    let reader = std::io::BufReader::new(file);

    let quotes = serde_json::from_reader::<_, Vec<Quote>>(reader)?
        .into_iter()
        .filter(|q| {
            let len = q.quote.chars().count();
            (MIN_QUOTE_LENGTH..=MAX_QUOTE_LENGTH).contains(&len)
        })
        .collect();
    Ok(quotes)
}

async fn insert_quotes(pool: &PgPool, items: Vec<Quote>) -> Result<u64> {
    let (authors, quotes): (Vec<String>, Vec<String>) = items
        .into_iter()
        .map(|q| (q.author.to_owned(), q.quote.to_owned()))
        .unzip();

    let res = sqlx::query(
        r#"
        INSERT INTO quotes (quote, author)
        SELECT * FROM UNNEST($1::text[], $2::text[])
        "#,
    )
    .bind(&quotes)
    .bind(&authors)
    .execute(pool)
    .await?;

    Ok(res.rows_affected())
}
