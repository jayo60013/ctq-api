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
