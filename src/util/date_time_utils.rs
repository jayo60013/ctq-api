use chrono::{Datelike, NaiveDate};

pub fn format_date_string(date: NaiveDate) -> String {
    let day = date.day();
    format!(
        "{} {}{}, {}",
        date.format("%B"),
        day,
        get_day_suffix(&day),
        date.year()
    )
}

fn get_day_suffix(day: &u32) -> &str {
    match day {
        11..=13 => "th",
        _ => match day % 10 {
            1 => "st",
            2 => "nd",
            3 => "rd",
            _ => "th",
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case("2025-01-01", "January 1st, 2025")]
    #[test_case("2026-02-02", "February 2nd, 2026")]
    #[test_case("2027-03-03", "March 3rd, 2027")]
    #[test_case("2028-04-04", "April 4th, 2028")]
    #[test_case("2029-05-11", "May 11th, 2029")]
    fn test_format_date_string(input: &str, expected: &str) {
        // Given
        let date = NaiveDate::parse_from_str(input, "%Y-%m-%d").unwrap();

        // When
        let actual_date_string = format_date_string(date);

        // Then
        assert_eq!(actual_date_string, expected);
    }
}
