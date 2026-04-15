use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct QuoteRow {
    pub author: String,
    pub source: Option<String>,
    pub quote: String,
}
