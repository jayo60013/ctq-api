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

/// Validates if a user can perform an action based on their current budget
/// Returns error if insufficient budget
pub fn validate_budget(
    current_checks_used: i32,
    current_solves_used: i32,
    action_cost: i32,
) -> Result<(), ApiError> {
    const MAX_BUDGET: i32 = 10;
    let current_total = current_checks_used + (current_solves_used * 2);
    let remaining_budget = MAX_BUDGET - current_total;

    if remaining_budget < action_cost {
        return Err(ApiError::ValidationError(format!(
            "Insufficient assist budget. Current usage: {current_total} / 10, action cost: {action_cost}, remaining: {remaining_budget}"
        )));
    }

    Ok(())
}
