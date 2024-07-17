use std::collections::HashSet;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::time::Instant;

use chrono::{DateTime, Local, Utc};
use hyper::{Body, Request, Response, StatusCode, Uri};

use crate::state::{HistogramMap, HttpClient, Log, LogList};

#[allow(clippy::too_many_arguments)]
pub async fn proxy(
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

    let mut proxied_req =
        Request::builder().method(req_method.clone()).uri(uri).body(req.into_body()).unwrap();

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
    histograms.entry("Overall".to_string()).or_default().add(duration, timestamp);

    histograms.entry(req_uri.path().to_string()).or_default().add(duration, timestamp);

    Ok(resp)
}
