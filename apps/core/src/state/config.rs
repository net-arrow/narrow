use std::net::IpAddr;

pub struct Config {
    /// The port number to run the proxy server on
    #[allow(dead_code)]
    pub proxy: u16,

    /// The interval in seconds to print the histograms
    #[allow(dead_code)]
    pub interval: u64,

    /// The host of the target server
    #[allow(dead_code)]
    pub host: String,

    /// The port of the target server
    #[allow(dead_code)]
    pub port: u16,

    /// Blacklisted IP addresses (comma-separated)
    #[allow(dead_code)]
    pub blacklist: Vec<IpAddr>,

    /// Whether to send the histograms to a monitoring server
    #[allow(dead_code)]
    pub monitoring: bool,

    /// The host of the monitoring server
    #[allow(dead_code)]
    pub server: String,

    /// The key to authenticate with the monitoring server
    #[allow(dead_code)]
    pub key: String,
}

// unit test
#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_config() {
        let config = Config {
            proxy: 8001,
            interval: 30,
            host: "example.com".to_string(),
            port: 3001,
            blacklist: vec![],
            monitoring: false,
            server: "https://monitoring.narrow.so".to_string(),
            key: "".to_string(),
        };

        assert_eq!(config.proxy, 8001);
        assert_eq!(config.interval, 30);
        assert_eq!(config.host, "example.com");
        assert_eq!(config.port, 3001);
        assert_eq!(config.blacklist, vec![] as Vec<IpAddr>);
        assert_eq!(config.monitoring, false);
        assert_eq!(config.server, "https://monitoring.narrow.so");
        assert_eq!(config.key, "");
    }
}
