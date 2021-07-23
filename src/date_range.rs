use chrono::{Date, Datelike, Local};
use rusoto_ce::DateInterval;

#[derive(Debug, PartialEq)]
pub struct ReportDateRange {
    start_date: Date<Local>,
    end_date: Date<Local>,
}
impl ReportDateRange {
    pub fn new(reporting_date: Date<Local>) -> Self {
        let first_day_of_month = reporting_date.with_day(1).unwrap();

        let start_date: Date<Local>;
        if reporting_date == first_day_of_month {
            // First day of the previous month
            start_date = first_day_of_month.pred().with_day(1).unwrap();
        } else {
            start_date = first_day_of_month;
        }

        ReportDateRange {
            start_date: start_date,
            end_date: reporting_date,
        }
    }

    pub fn as_date_interval(self) -> DateInterval {
        DateInterval {
            end: self.end_date.format("%Y-%m-%d").to_string(),
            start: self.start_date.format("%Y-%m-%d").to_string(),
        }
    }
}

#[cfg(test)]
mod date_range_tests {
    use super::*;
    use chrono::{Local, TimeZone};
    use rusoto_ce::DateInterval;

    #[test]
    fn reporting_in_middle_of_month() {
        let input_date = Local.ymd(2021, 7, 18);

        let expected_date_range = ReportDateRange {
            start_date: Local.ymd(2021, 7, 1),
            end_date: Local.ymd(2021, 7, 18),
        };

        let actual_date_range = ReportDateRange::new(input_date);

        assert_eq!(expected_date_range, actual_date_range);
    }

    #[test]
    fn reporting_at_beginning_of_month() {
        let input_date = Local.ymd(2021, 7, 1);

        let expected_date_range = ReportDateRange {
            start_date: Local.ymd(2021, 6, 1),
            end_date: Local.ymd(2021, 7, 1),
        };

        let actual_date_range = ReportDateRange::new(input_date);

        assert_eq!(expected_date_range, actual_date_range);
    }

    #[test]
    fn convert_into_date_interval_correctly() {
        let input_date_range = ReportDateRange {
            start_date: Local.ymd(2021, 7, 1),
            end_date: Local.ymd(2021, 7, 22),
        };

        let expected_date_interval = DateInterval {
            start: "2021-07-01".to_string(),
            end: "2021-07-22".to_string(),
        };

        let actual_date_interval = input_date_range.as_date_interval();

        assert_eq!(expected_date_interval, actual_date_interval);
    }
}
