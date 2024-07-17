mod config;
mod net;
mod state;
mod statistics;

use std::collections::{HashMap, HashSet};
use std::convert::Infallible;
use std::net::{IpAddr, SocketAddr};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use clap::Parser;
use hyper::server::conn::AddrStream;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Client, Server};
use tokio::time;

use crate::config::Args;
use crate::net::proxy::proxy;
use crate::state::{Config, HistogramMap, LogList};
use crate::statistics::print_histograms;

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let config = Config {
        blacklist: args.blacklist.clone(),
        host: args.host.clone(),
        interval: args.interval,
        key: args.key.clone(),
        monitoring: args.monitoring,
        port: args.port,
        proxy: args.proxy,
        server: args.server.clone(),
    };

    let addr = SocketAddr::from(([127, 0, 0, 1], config.proxy));
    let client = Client::new();

    // Create shared state for the histograms and log list
    let histograms: HistogramMap = Arc::new(Mutex::new(HashMap::new()));
    let loglist: LogList = Arc::new(Mutex::new(Vec::new()));
    let blacklist: Arc<HashSet<IpAddr>> = Arc::new(config.blacklist.clone().into_iter().collect());

    let histograms_for_timer = Arc::clone(&histograms);
    let loglist_for_timer = Arc::clone(&loglist);

    tokio::spawn(async move {
        // Wait for the first period before starting the timer
        time::sleep(Duration::from_secs(config.interval)).await;

        let mut interval = time::interval(Duration::from_secs(config.interval));
        loop {
            interval.tick().await;
            let histograms = histograms_for_timer.lock().unwrap().clone();
            print_histograms(&histograms);

            // TODO: send the histograms and loglist to a monitoring service

            histograms_for_timer.lock().unwrap().clear();
            loglist_for_timer.lock().unwrap().clear();
        }
    });

    let target_host = config.host.clone();
    let target_port = config.port;

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
    println!("Forwarding traffic to http://{}:{}", config.host, config.port);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}
