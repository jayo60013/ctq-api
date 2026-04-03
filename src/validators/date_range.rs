use chrono::NaiveDate;
use crate::error::ApiError;

#[derive(Debug, Clone)]
pub struct DateRange {
    pub from: NaiveDate,
    pub to: NaiveDate,
}

impl DateRange {
    pub fn new(from_str: &str, to_str: &str) -> Result<Self, ApiError> {
        let from = NaiveDate::parse_from_str(from_str, "%Y-%m-%d")
            .map_err(|_| ApiError::ValidationError(
                format!("'from' parameter must be a valid date in YYYY-MM-DD format, got: {from_str}")
            ))?;

        let to = NaiveDate::parse_from_str(to_str, "%Y-%m-%d")
            .map_err(|_| ApiError::ValidationError(
                format!("'to' parameter must be a valid date in YYYY-MM-DD format, got: {to_str}")
            ))?;

        Ok(DateRange { from, to })
    }
}

