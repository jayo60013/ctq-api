use crate::error::ApiError;

/// Represents a user's assist budget for puzzle-solving.
///
/// Budget system:
/// - Each check costs 1 point
/// - Each solve costs 2 points
/// - Maximum total budget: 10 points
#[derive(Debug, Clone, Copy)]
pub struct Budget {
    pub checks_used: i32,
    pub solves_used: i32,
}

impl Budget {
    /// Creates a new Budget from current usage
    pub fn new(checks_used: i32, solves_used: i32) -> Self {
        Budget {
            checks_used,
            solves_used,
        }
    }

    /// Maximum allowed budget (in points)
    const MAX_BUDGET: i32 = 10;

    /// Cost of a single letter solve (in points)
    const SOLVE_COST: i32 = 2;

    /// Calculates the current total budget spent
    pub fn total_used(self) -> i32 {
        self.checks_used + (self.solves_used * Self::SOLVE_COST)
    }

    /// Calculates the remaining budget available
    pub fn remaining(self) -> i32 {
        Self::MAX_BUDGET - self.total_used()
    }

    /// Validates if an action with the given cost is allowed
    pub fn validate_action(self, action_cost: i32) -> Result<(), ApiError> {
        if self.remaining() < action_cost {
            return Err(ApiError::ValidationError(format!(
                "Insufficient assist budget. Current usage: {} / {}, action cost: {}, remaining: {}",
                self.total_used(),
                Self::MAX_BUDGET,
                action_cost,
                self.remaining()
            )));
        }
        Ok(())
    }

    /// Validates a check action (costs 1 point)
    #[allow(dead_code)]
    pub fn validate_check(self) -> Result<(), ApiError> {
        self.validate_action(1)
    }

    /// Validates a solve action (costs 2 points)
    #[allow(dead_code)]
    pub fn validate_solve(self) -> Result<(), ApiError> {
        self.validate_action(Self::SOLVE_COST)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_budget_total_used() {
        let budget = Budget::new(2, 3);
        assert_eq!(budget.total_used(), 8); // 2 + (3 * 2)
    }

    #[test]
    fn test_budget_remaining() {
        let budget = Budget::new(2, 3);
        assert_eq!(budget.remaining(), 2); // 10 - 8
    }

    #[test]
    fn test_validate_check_sufficient() {
        let budget = Budget::new(2, 3);
        assert!(budget.validate_check().is_ok());
    }

    #[test]
    fn test_validate_check_insufficient() {
        let budget = Budget::new(10, 0);
        assert!(budget.validate_check().is_err());
    }

    #[test]
    fn test_validate_solve_sufficient() {
        let budget = Budget::new(8, 0);
        assert!(budget.validate_solve().is_ok());
    }

    #[test]
    fn test_validate_solve_insufficient() {
        let budget = Budget::new(8, 1);
        assert!(budget.validate_solve().is_err());
    }
}
