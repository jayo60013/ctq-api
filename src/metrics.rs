use prometheus::{IntCounterVec, Opts};
use std::sync::LazyLock;

pub static PUZZLE_SOLVED_TOTAL: LazyLock<IntCounterVec> = LazyLock::new(|| {
    let opts = Opts::new("puzzle_solved_total", "Total number of puzzles solved");
    IntCounterVec::new(opts, &["puzzle_type", "user_type"]).unwrap()
});

pub static LETTER_CHECKS_TOTAL: LazyLock<IntCounterVec> = LazyLock::new(|| {
    let opts = Opts::new(
        "puzzle_letter_checks_total",
        "Total number of letter checks",
    );
    IntCounterVec::new(opts, &["puzzle_type", "is_correct"]).unwrap()
});

pub static LETTER_SOLVES_TOTAL: LazyLock<IntCounterVec> = LazyLock::new(|| {
    let opts = Opts::new(
        "puzzle_letter_solves_total",
        "Total number of letter solves revealed",
    );
    IntCounterVec::new(opts, &["puzzle_type"]).unwrap()
});

pub fn increment_puzzle_solved(puzzle_type: &str, user_type: &str) {
    PUZZLE_SOLVED_TOTAL
        .with_label_values(&[puzzle_type, user_type])
        .inc();
}

#[allow(dead_code)]
pub fn register_custom_metrics(registry: &prometheus::Registry) {
    registry
        .register(Box::new(PUZZLE_SOLVED_TOTAL.clone()))
        .unwrap();
    registry
        .register(Box::new(LETTER_CHECKS_TOTAL.clone()))
        .unwrap();
    registry
        .register(Box::new(LETTER_SOLVES_TOTAL.clone()))
        .unwrap();
}

#[cfg(test)]
mod tests {
    use super::PUZZLE_SOLVED_TOTAL;

    #[test]
    fn puzzle_solved_metric_accepts_two_labels() {
        assert!(PUZZLE_SOLVED_TOTAL
            .get_metric_with_label_values(&["daily", "guest"])
            .is_ok());
    }

    #[test]
    fn puzzle_solved_metric_rejects_wrong_label_count() {
        assert!(PUZZLE_SOLVED_TOTAL
            .get_metric_with_label_values(&["daily"])
            .is_err());
    }
}
