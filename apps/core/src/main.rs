use chrono::{DateTime, Local, Utc};
use std::collections::{HashMap, HashSet};
use std::convert::Infallible;
use std::net::{IpAddr, SocketAddr};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use clap::Parser;
use hyper::server::conn::AddrStream;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Client, Method, Request, Response, Server, StatusCode, Uri};
use prettytable::{format, Cell, Row, Table};
use tokio::time;

type HttpClient = Client<hyper::client::HttpConnector>;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = "An observation tool to better monitor and secure your web traffic.")]
struct Args {
    /// The port number to run the proxy server on
    #[clap(short, long, default_value = "8000")]
    proxy: u16,

    /// The interval in seconds to print the histograms
    #[clap(short, long, default_value = "60")]
    interval: u64,

    /// The host of the target server
    #[clap(short = 'H', long, default_value = "localhost")]
    host: String,

    /// The port of the target server
    #[clap(short = 'P', long, default_value = "3000")]
    port: u16,

    /// Blacklisted IP addresses (comma-separated)
    #[clap(short, long, use_value_delimiter = true, value_delimiter = ',')]
    blacklist: Vec<IpAddr>,

    /// Whether to send the histograms to a monitoring server
    #[clap(short, long, default_value = "false")]
    monitoring: bool,

    /// The host of the monitoring server
    #[clap(short, long, default_value = "https://monitoring.narrow.so")]
    server: String,

    /// The key to authenticate with the monitoring server
    #[clap(short, long, default_value = None)]
    key: String,
}

#[derive(Debug, Default, Clone)]
struct Log {
    timestamp: DateTime<Utc>,
    req_method: Method,
    req_uri: String,
    requester_ip: String,
    micros: u128,
}

#[derive(Debug, Default, Clone)]
struct Histogram {
    count_0_10: u64,
    count_11_100: u64,
    count_101_250: u64,
    count_251_500: u64,
    count_501_1000: u64,
    count_1000_plus: u64,
    total_requests: u64,
    last_request_time: Option<DateTime<Utc>>,
}

impl Histogram {
    fn add(&mut self, duration: Duration, timestamp: DateTime<Utc>) {
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

type HistogramMap = Arc<Mutex<HashMap<String, Histogram>>>;
type LogList = Arc<Mutex<Vec<Log>>>;

#[allow(clippy::too_many_arguments)]
async fn proxy(
    client: HttpClient,
    req: Request<Body>,
    requester_ip: SocketAddr,
    histograms: HistogramMap,
    loglist: LogList,
    target_host: String,
    target_port: u16,
    blacklist: Arc<HashSet<IpAddr>>,
) -> Result<Response<Body>, hyper::Error> {
    let timestamp = Utc::now();

    let local_time: DateTime<Local> = DateTime::from(timestamp);

    if blacklist.contains(&requester_ip.ip()) {
        println!("Rejected blacklisted IP: {}", requester_ip.ip());
        return Ok(Response::builder()
            .status(StatusCode::FORBIDDEN)
            .body(Body::from("Access denied"))
            .unwrap());
    }

    let start = Instant::now();

    let req_method = req.method().clone();
    let req_uri = req.uri().clone();
    let req_headers = req.headers().clone();

    let uri = format!(
        "http://{}:{}{}",
        target_host,
        target_port,
        req_uri.path_and_query().map(|x| x.as_str()).unwrap_or("")
    )
    .parse::<Uri>()
    .unwrap();

    let mut proxied_req = Request::builder()
        .method(req_method.clone())
        .uri(uri)
        .body(req.into_body())
        .unwrap();

    *proxied_req.headers_mut() = req_headers;

    let resp = client.request(proxied_req).await?;

   

    let duration = start.elapsed();
    println!(
        "{} {} {} - From: {} - Response time: {:?}",
        local_time.format("%Y-%m-%d %H:%M:%S %Z"),
        req_method,
        req_uri,
        requester_ip,
        duration
    );

    loglist.lock().unwrap().push(Log {
        timestamp,
        req_method,
        req_uri: req_uri.to_string(),
        requester_ip: requester_ip.ip().to_string(),
        micros: duration.as_micros(),
    });

    let mut histograms = histograms.lock().unwrap();
    histograms
        .entry("Overall".to_string())
        .or_default()
        .add(duration, timestamp);

    histograms
        .entry(req_uri.path().to_string())
        .or_default()
        .add(duration, timestamp);

    Ok(resp)
}

fn add_histogram_row(table: &mut Table, endpoint: &str, hist: &Histogram) {
    let last_request = hist
        .last_request_time
        .map(|t| {
            DateTime::<Local>::from(t)
                .format("%Y-%m-%d %H:%M:%S %Z")
                .to_string()
        })
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

fn print_histograms(histograms: &HashMap<String, Histogram>) {
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
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let addr = SocketAddr::from(([127, 0, 0, 1], args.proxy));
    let client = Client::new();

    // Create shared state for the histograms and log list
    let histograms: HistogramMap = Arc::new(Mutex::new(HashMap::new()));
    let loglist: LogList = Arc::new(Mutex::new(Vec::new()));
    let blacklist: Arc<HashSet<IpAddr>> = Arc::new(args.blacklist.clone().into_iter().collect());

    let histograms_for_timer = Arc::clone(&histograms);
    let loglist_for_timer = Arc::clone(&loglist);
    tokio::spawn(async move {
        // Wait for the first period before starting the timer
        time::sleep(Duration::from_secs(args.interval)).await;

        let mut interval = time::interval(Duration::from_secs(args.interval));
        loop {
            interval.tick().await;
            let histograms = histograms_for_timer.lock().unwrap().clone();
            print_histograms(&histograms);

            // TODO: send the histograms and loglist to a monitoring service

            histograms_for_timer.lock().unwrap().clear();
            loglist_for_timer.lock().unwrap().clear();
        }
    });

    let target_host = args.host.clone();
    let target_port = args.port;

    let make_svc = make_service_fn(move |conn: &AddrStream| {
        let client = client.clone();
        let requester_ip = conn.remote_addr();
        let histograms = Arc::clone(&histograms);
        let loglist = Arc::clone(&loglist);
        let target_host = target_host.clone();
        let target_port = target_port;
        let blacklist = Arc::clone(&blacklist);

        async move {
            Ok::<_, Infallible>(service_fn(move |req| {
                proxy(
                    client.clone(),
                    req,
                    requester_ip,
                    Arc::clone(&histograms),
                    Arc::clone(&loglist),
                    target_host.clone(),
                    target_port,
                    Arc::clone(&blacklist),
                )
            }))
        }
    });

    let server = Server::bind(&addr).serve(make_svc);

    println!("Proxy server running on http://{}", addr);
    println!("Forwarding traffic to http://{}:{}", args.host, args.port);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}
