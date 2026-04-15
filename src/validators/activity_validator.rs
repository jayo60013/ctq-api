use crate::error::ApiError;
use crate::models::ActivityUpdateRequest;

pub fn validate_activity_request(req: &ActivityUpdateRequest) -> Result<(), ApiError> {
    let total_cost = req.checks_used + (req.solves_used * 2);

    if total_cost > 10 {
        return Err(ApiError::ValidationError(format!(
            "checks_used ({}) + solves_used * 2 ({}) must be <= 10, got {}",
            req.checks_used,
            req.solves_used * 2,
            total_cost
        )));
    }

    Ok(())
}
