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
            quote TEXT NOT NULL,
            source TEXT
        )
        "#,
    )
    .execute(pool)
    .await
    .context("Failed to create quotes table")?;

    sqlx::query("ALTER TABLE quotes ADD COLUMN IF NOT EXISTS source TEXT")
        .execute(pool)
        .await
        .context("Failed to update quotes table schema")?;

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
    let mut quotes = Vec::with_capacity(items.len());
    let mut authors = Vec::with_capacity(items.len());
    let mut sources = Vec::with_capacity(items.len());

    for q in items {
        quotes.push(q.quote);
        authors.push(q.author);
        sources.push(q.source);
    }

    let res = sqlx::query(
        r#"
        INSERT INTO quotes (quote, author, source)
        SELECT * FROM UNNEST($1::text[], $2::text[], $3::text[])
        "#,
    )
    .bind(&quotes)
    .bind(&authors)
    .bind(&sources)
    .execute(pool)
    .await?;

    Ok(res.rows_affected())
}
