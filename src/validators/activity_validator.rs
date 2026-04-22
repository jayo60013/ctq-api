use crate::error::ApiError;
use crate::models::Budget;

/// Validates if a user can perform an action based on their current budget.
/// Returns error if insufficient budget.
///
/// Wraps the Budget struct validation for backward compatibility.
pub fn validate_budget(
    current_checks_used: i32,
    current_solves_used: i32,
    action_cost: i32,
) -> Result<(), ApiError> {
    let budget = Budget::new(current_checks_used, current_solves_used);
    budget.validate_action(action_cost)
}
