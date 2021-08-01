use chrono::{Date, DateTime, Datelike, TimeZone};
use chrono_tz::Tz;
use rusoto_ce::DateInterval;
use std::error;
use std::fmt::Display;

pub fn date_in_specified_timezone<T: TimeZone>(
    datetime: DateTime<T>,
    tz_string: String,
) -> Result<Date<Tz>, Box<dyn error::Error>> {
    let timezone: Result<Tz, _> = tz_string.parse();
    match timezone {
        Ok(timezone) => Ok(datetime.with_timezone(&timezone).date()),
        Err(e) => Err(format!("Invalid Timezone!: {}", e).into()),
    }
}

#[cfg(test)]
mod test_date_with_timezone {
    use super::date_in_specified_timezone;
    use chrono::{Local, TimeZone, Utc};

    #[test]
    fn convert_timezone_correctly() {
        let input_datetime = Local
            .datetime_from_str("2021-07-31 12:00:00 UTC", "%Y-%m-%d %H:%M:%S %Z")
            .unwrap();

        let tz_string = "Asia/Tokyo".to_string();

        let actual_date = date_in_specified_timezone(input_datetime, tz_string).unwrap();

        assert_eq!("2021-07-31JST", format!("{}", actual_date));
    }

    #[test]
    fn with_different_date() {
        let input_datetime = Utc
            .datetime_from_str("2021-07-31 15:00:00", "%Y-%m-%d %H:%M:%S")
            .unwrap();

        let tz_string = "Asia/Tokyo".to_string();

        let actual_date = date_in_specified_timezone(input_datetime, tz_string).unwrap();

        assert_eq!("2021-08-01JST", format!("{}", actual_date));
    }

    #[test]
    fn return_error_for_invalid_timezone() {
        let input_datetime = Local
            .datetime_from_str("2021-07-31 15:05:00 UTC", "%Y-%m-%d %H:%M:%S %Z")
            .unwrap();

        let tz_string = "Invalid/Timezone".to_string();

        let actual_date = date_in_specified_timezone(input_datetime, tz_string);

        assert!(actual_date.is_err());
    }
}

#[derive(Debug)]
pub struct ReportDateRange<T>
where
    T: TimeZone,
    <T as TimeZone>::Offset: Display,
{
    start_date: Date<T>,
    end_date: Date<T>,
}
impl<T> ReportDateRange<T>
where
    T: TimeZone,
    <T as TimeZone>::Offset: Display,
{
    pub fn new(reporting_date: Date<T>) -> Self {
        let first_day_of_month = reporting_date.with_day(1).unwrap();

        let start_date: Date<T>;
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

    pub fn as_date_interval(&self) -> DateInterval {
        DateInterval {
            end: self.end_date.format("%Y-%m-%d").to_string(),
            start: self.start_date.format("%Y-%m-%d").to_string(),
        }
    }
}
impl<T> PartialEq for ReportDateRange<T>
where
    T: TimeZone,
    <T as TimeZone>::Offset: Display,
{
    fn eq(&self, other: &ReportDateRange<T>) -> bool {
        self.start_date == other.start_date && self.end_date == other.end_date
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
