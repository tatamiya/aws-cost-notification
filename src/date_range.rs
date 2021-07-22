use chrono::{Date, Datelike, Local};

#[derive(Debug, PartialEq)]
pub struct DateRange {
    pub start_date: Date<Local>,
    pub end_date: Date<Local>,
}

pub fn get_date_range(target_date: Date<Local>) -> DateRange {
    let first_day_of_month = target_date.with_day(1).unwrap();

    let start_date: Date<Local>;
    if target_date == first_day_of_month {
        // First day of the previous month
        start_date = first_day_of_month.pred().with_day(1).unwrap();
    } else {
        start_date = first_day_of_month;
    }

    DateRange {
        start_date: start_date,
        end_date: target_date,
    }
}

#[cfg(test)]
mod date_range_tests {
    use super::*;
    use chrono::{Local, TimeZone};

    #[test]
    fn middle_of_month() {
        let input_date = Local.ymd(2021, 7, 18);

        let expected_date_range = DateRange {
            start_date: Local.ymd(2021, 7, 1),
            end_date: Local.ymd(2021, 7, 18),
        };

        let actual_date_range = get_date_range(input_date);

        assert_eq!(expected_date_range, actual_date_range);
    }

    #[test]
    fn beginning_of_month() {
        let input_date = Local.ymd(2021, 7, 1);

        let expected_date_range = DateRange {
            start_date: Local.ymd(2021, 6, 1),
            end_date: Local.ymd(2021, 7, 1),
        };

        let actual_date_range = get_date_range(input_date);

        assert_eq!(expected_date_range, actual_date_range);
    }
}
