use std::collections::HashMap;
use std::time::Duration;

use chrono::{DateTime, Local, Utc};
use prettytable::{format, Cell, Row, Table};

#[derive(Debug, Default, Clone)]
pub struct Histogram {
    pub count_0_10: u64,
    pub count_11_100: u64,
    pub count_101_250: u64,
    pub count_251_500: u64,
    pub count_501_1000: u64,
    pub count_1000_plus: u64,
    pub total_requests: u64,
    pub last_request_time: Option<DateTime<Utc>>,
}

impl Histogram {
    pub fn add(&mut self, duration: Duration, timestamp: DateTime<Utc>) {
        let ms = duration.as_millis();
        match ms {
            0..=10 => self.count_0_10 += 1,
            11..=100 => self.count_11_100 += 1,
            101..=250 => self.count_101_250 += 1,
            251..=500 => self.count_251_500 += 1,
            501..=1000 => self.count_501_1000 += 1,
            _ => self.count_1000_plus += 1,
        }

        self.total_requests += 1;
        self.last_request_time = Some(timestamp);
    }
}

pub fn add_histogram_row(table: &mut Table, endpoint: &str, hist: &Histogram) {
    let last_request = hist
        .last_request_time
        .map(|t| DateTime::<Local>::from(t).format("%Y-%m-%d %H:%M:%S %Z").to_string())
        .unwrap_or_else(|| "N/A".to_string());

    table.add_row(Row::new(vec![
        Cell::new(endpoint),
        Cell::new(&hist.count_0_10.to_string()),
        Cell::new(&hist.count_11_100.to_string()),
        Cell::new(&hist.count_101_250.to_string()),
        Cell::new(&hist.count_251_500.to_string()),
        Cell::new(&hist.count_501_1000.to_string()),
        Cell::new(&hist.count_1000_plus.to_string()),
        Cell::new(&hist.total_requests.to_string()),
        Cell::new(&last_request),
    ]));
}

pub fn print_histograms(histograms: &HashMap<String, Histogram>) -> String {
    // Print a newline before the histogram
    println!("\nResponse Time Histogram:");

    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);
    table.set_titles(Row::new(vec![
        Cell::new("Endpoint"),
        Cell::new("0-10ms"),
        Cell::new("11-100ms"),
        Cell::new("101-250ms"),
        Cell::new("251-500ms"),
        Cell::new("501-1000ms"),
        Cell::new("1000ms+"),
        Cell::new("Total"),
        Cell::new("Last Request"),
    ]));

    if histograms.is_empty() || (histograms.len() == 1 && histograms.contains_key("Overall")) {
        add_histogram_row(&mut table, "Overall", &Histogram::default());
    } else {
        if let Some(overall_hist) = histograms.get("Overall") {
            add_histogram_row(&mut table, "Overall", overall_hist);
        }

        for (endpoint, hist) in histograms.iter() {
            if endpoint != "Overall" {
                add_histogram_row(&mut table, endpoint, hist);
            }
        }
    }

    table.printstd();

    // Print a newline after the histogram
    println!();

    table.to_string()
}

// unit test
#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_histogram() {
        let mut hist = Histogram::default();
        let timestamp = Utc::now();

        hist.add(Duration::from_millis(5), timestamp);
        hist.add(Duration::from_millis(50), timestamp);
        hist.add(Duration::from_millis(150), timestamp);
        hist.add(Duration::from_millis(300), timestamp);
        hist.add(Duration::from_millis(600), timestamp);
        hist.add(Duration::from_millis(1200), timestamp);

        assert_eq!(hist.count_0_10, 1);
        assert_eq!(hist.count_11_100, 1);
        assert_eq!(hist.count_101_250, 1);
        assert_eq!(hist.count_251_500, 1);
        assert_eq!(hist.count_501_1000, 1);
        assert_eq!(hist.count_1000_plus, 1);
        assert_eq!(hist.total_requests, 6);
        assert_eq!(hist.last_request_time, Some(timestamp));
    }

    #[test]
    fn test_add_histogram_row() {
        let mut table = Table::new();
        table.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);
        table.set_titles(Row::new(vec![
            Cell::new("Endpoint"),
            Cell::new("0-10ms"),
            Cell::new("11-100ms"),
            Cell::new("101-250ms"),
            Cell::new("251-500ms"),
            Cell::new("501-1000ms"),
            Cell::new("1000ms+"),
            Cell::new("Total"),
            Cell::new("Last Request"),
        ]));

        let hist = Histogram {
            count_0_10: 1,
            count_11_100: 2,
            count_101_250: 3,
            count_251_500: 4,
            count_501_1000: 5,
            count_1000_plus: 6,
            total_requests: 21,
            last_request_time: Some(Utc::now()),
        };

        add_histogram_row(&mut table, "test", &hist);

        let binding = DateTime::<Local>::from(hist.last_request_time.unwrap())
            .format("%Y-%m-%d %H:%M:%S %Z")
            .to_string();
        let expected = vec!["test", "1", "2", "3", "4", "5", "6", "21", &binding];

        assert_eq!(
            table.get_row(0).unwrap().into_iter().map(|c| c.to_string()).collect::<Vec<_>>(),
            expected
        );
    }

    #[test]
    fn test_print_histograms() {
        let mut histograms = HashMap::new();
        histograms.insert("Overall".to_string(), Histogram::default());
        histograms.insert(
            "/test".to_string(),
            Histogram {
                count_0_10: 1,
                count_11_100: 2,
                count_101_250: 3,
                count_251_500: 4,
                count_501_1000: 5,
                count_1000_plus: 6,
                total_requests: 21,
                last_request_time: None,
            },
        );

        let table = print_histograms(&histograms);

        let expected = vec![
            vec![
                "Endpoint",
                "0-10ms",
                "11-100ms",
                "101-250ms",
                "251-500ms",
                "501-1000ms",
                "1000ms+",
                "Total",
                "Last",
                "Request",
            ],
            vec!["Overall", "0", "0", "0", "0", "0", "0", "0", "N/A"],
            vec!["/test", "1", "2", "3", "4", "5", "6", "21", "N/A"],
        ];

        let mut i: usize = 0;
        for row in table.lines() {
            // skip looping if the row if the row is a separator -----
            if row.contains("-----") {
                continue;
            }

            let cells = row
                .split_whitespace()
                .filter(|c| c.to_string() != "|")
                .map(|c| c.to_string())
                .collect::<Vec<_>>();

            assert_eq!(cells, expected[i]);
            i += 1;
        }
    }
}
