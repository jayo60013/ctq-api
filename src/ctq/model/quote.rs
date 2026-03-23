use serde::Deserialize;
use sqlx::prelude::FromRow;

#[derive(Debug, Deserialize, FromRow)]
pub struct Quote {
    #[serde(rename = "Quote")]
    pub quote: String,

    #[serde(rename = "Author")]
    pub author: String,
}
