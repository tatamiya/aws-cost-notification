use chrono::{Date, Local, NaiveDate, TimeZone};
use rusoto_ce::{GetCostAndUsageResponse, Group, MetricValue};

/// AWS Cost
#[derive(Debug, PartialEq, Clone, PartialOrd)]
pub struct Cost {
    pub amount: f32,
    pub unit: String,
}
impl From<MetricValue> for Cost {
    fn from(from: MetricValue) -> Cost {
        let parsed_amount = from.amount.as_ref().unwrap().parse::<f32>().unwrap();

        let parsed_unit = from.unit.as_ref().unwrap().to_string();

        Cost {
            amount: parsed_amount,
            unit: parsed_unit,
        }
    }
}

/// Period of cost aggregation in the API response.
#[derive(Debug, PartialEq)]
pub struct ReportedDateRange {
    pub start_date: Date<Local>,
    pub end_date: Date<Local>,
}

/// Total AWS cost during `date_range`.
#[derive(Debug, PartialEq)]
pub struct TotalCost {
    pub date_range: ReportedDateRange,
    pub cost: Cost,
}
impl From<GetCostAndUsageResponse> for TotalCost {
    /// Parse the API response into `TotalCost`
    fn from(from: GetCostAndUsageResponse) -> TotalCost {
        let result_by_time = &from.results_by_time.as_ref().unwrap()[0];
        let time_period = result_by_time.time_period.as_ref().unwrap();

        let parsed_start_date = parse_timestamp_into_local_date(&time_period.start).unwrap();
        let parsed_end_date = parse_timestamp_into_local_date(&time_period.end).unwrap();

        let amortized_cost = result_by_time
            .total
            .as_ref()
            .unwrap()
            .get("AmortizedCost")
            .unwrap()
            .clone();

        TotalCost {
            date_range: ReportedDateRange {
                start_date: parsed_start_date,
                end_date: parsed_end_date,
            },
            cost: amortized_cost.into(),
        }
    }
}

/// Parse the timestamp in the `time_period` field of the API response.
fn parse_timestamp_into_local_date(timestamp: &str) -> chrono::LocalResult<Date<Local>> {
    let parsed_start_date = NaiveDate::parse_from_str(timestamp, "%Y-%m-%d")
        .ok()
        .unwrap();
    Local.from_local_date(&parsed_start_date)
}

/// The cost of a service.
#[derive(Debug, PartialEq, Clone)]
pub struct ServiceCost {
    pub service_name: String,
    pub cost: Cost,
}
impl From<Group> for ServiceCost {
    /// Parse `Group` in the API response into ServiceCost.
    fn from(from: Group) -> ServiceCost {
        let service_name = &from.keys.as_ref().unwrap()[0];
        let amortized_cost = from
            .metrics
            .as_ref()
            .unwrap()
            .get("AmortizedCost")
            .unwrap()
            .clone();

        ServiceCost {
            service_name: service_name.to_string(),
            cost: amortized_cost.into(),
        }
    }
}
impl ServiceCost {
    /// Parse the API response into a vector of `ServiceCost`
    pub fn from_response(res: &GetCostAndUsageResponse) -> Vec<Self> {
        let result_by_time = &res.results_by_time.as_ref().unwrap()[0];
        let groups = result_by_time.groups.as_ref().unwrap();
        groups.iter().map(|x| x.clone().into()).collect()
    }
}

#[cfg(test)]
mod test_parsers {

    use super::*;
    use rusoto_ce::*;

    use crate::cost_explorer::test_utils::{prepare_sample_response, InputServiceCost};

    #[test]
    fn parse_timestamp_into_local_date_correctly() {
        let input_timestamp = "2021-07-22";
        let expected_parsed_date = Local.ymd(2021, 7, 22);

        let actual_parsed_date = parse_timestamp_into_local_date(input_timestamp).unwrap();
        assert_eq!(expected_parsed_date, actual_parsed_date);
    }

    #[test]
    fn parse_cost_from_metric_value_correctly() {
        let input_metric_value = MetricValue {
            amount: Some("123.56".to_string()),
            unit: Some("USD".to_string()),
        };

        let expected_cost = Cost {
            amount: 123.56,
            unit: "USD".to_string(),
        };

        let actual_cost: Cost = input_metric_value.into();

        assert_eq!(expected_cost, actual_cost);
    }

    #[test]
    fn parse_total_cost_correctly() {
        let input_response: GetCostAndUsageResponse = prepare_sample_response(
            Some(DateInterval {
                start: String::from("2021-07-01"),
                end: String::from("2021-07-18"),
            }),
            Some(String::from("1234.56")),
            None,
        );

        let expected_parsed_total_cost = TotalCost {
            date_range: ReportedDateRange {
                start_date: Local.ymd(2021, 7, 1),
                end_date: Local.ymd(2021, 7, 18),
            },
            cost: Cost {
                amount: 1234.56,
                unit: String::from("USD"),
            },
        };

        let actual_parsed_total_cost: TotalCost = input_response.into();

        assert_eq!(expected_parsed_total_cost, actual_parsed_total_cost);
    }

    #[test]
    fn parse_service_costs_correctly() {
        let input_response: GetCostAndUsageResponse = prepare_sample_response(
            None,
            None,
            Some(vec![
                InputServiceCost::new("Amazon Simple Storage Service", "1234.56"),
                InputServiceCost::new("Amazon Elastic Compute Cloud", "31415.92"),
            ]),
        );
        let expected_parsed_service_costs = vec![
            ServiceCost {
                service_name: String::from("Amazon Simple Storage Service"),
                cost: Cost {
                    amount: 1234.56,
                    unit: String::from("USD"),
                },
            },
            ServiceCost {
                service_name: String::from("Amazon Elastic Compute Cloud"),
                cost: Cost {
                    amount: 31415.92,
                    unit: String::from("USD"),
                },
            },
        ];
        let actual_parsed_service_costs = ServiceCost::from_response(&input_response);

        assert_eq!(expected_parsed_service_costs, actual_parsed_service_costs);
    }
}
