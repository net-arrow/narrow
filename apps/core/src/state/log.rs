use chrono::{DateTime, Utc};
use hyper::Method;

#[derive(Debug, Default, Clone)]
pub struct Log {
    #[allow(dead_code)]
    pub timestamp: DateTime<Utc>,

    #[allow(dead_code)]
    pub req_method: Method,

    #[allow(dead_code)]
    pub req_uri: String,

    #[allow(dead_code)]
    pub requester_ip: String,

    #[allow(dead_code)]
    pub micros: u128,
}

// unit test
#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_log() {
        let log = Log {
            timestamp: Utc::now(),
            req_method: Method::GET,
            req_uri: "/".to_string(),
            requester_ip: "1.1.1.1".to_owned(),
            micros: 100,
        };

        assert_eq!(log.req_method, Method::GET);
        assert_eq!(log.req_uri, "/");
        assert_eq!(log.requester_ip, "1.1.1.1");
        assert_eq!(log.micros, 100);
    }
}
