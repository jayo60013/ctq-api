use crate::error::ApiError;

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
