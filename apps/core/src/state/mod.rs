mod config;
mod log;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub use config::*;
use hyper::Client;
pub use log::*;

use crate::statistics::Histogram;

pub type HttpClient = Client<hyper::client::HttpConnector>;
pub type HistogramMap = Arc<Mutex<HashMap<String, Histogram>>>;
pub type LogList = Arc<Mutex<Vec<Log>>>;
