use std::net::IpAddr;

use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[clap(
    author,
    version,
    about,
    long_about = "An observation tool to better monitor and secure your web traffic."
)]
pub struct Args {
    /// The port number to run the proxy server on
    #[clap(short, long, default_value = "8000")]
    pub proxy: u16,

    /// The interval in seconds to print the histograms
    #[clap(short, long, default_value = "60")]
    pub interval: u64,

    /// The host of the target server
    #[clap(short = 'H', long, default_value = "localhost")]
    pub host: String,

    /// The port of the target server
    #[clap(short = 'P', long, default_value = "3000")]
    pub port: u16,

    /// Blacklisted IP addresses (comma-separated)
    #[clap(short, long, use_value_delimiter = true, value_delimiter = ',')]
    pub blacklist: Vec<IpAddr>,

    /// Whether to send the histograms to a monitoring server
    #[clap(short, long, default_value = "false")]
    pub monitoring: bool,

    /// The host of the monitoring server
    #[clap(short, long, default_value = "https://monitoring.narrow.so")]
    pub server: String,

    /// The key to authenticate with the monitoring server
    #[clap(short, long, default_value = "")]
    pub key: String,
}

// unit test
#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_args() {
        let args = Args::parse_from(&[
            "test",
            "--proxy",
            "8001",
            "--interval",
            "30",
            "--host",
            "example.com",
            "--port",
            "3001",
        ]);

        assert_eq!(args.proxy, 8001);
        assert_eq!(args.interval, 30);
        assert_eq!(args.host, "example.com");
        assert_eq!(args.port, 3001);
        assert_eq!(args.blacklist, vec![] as Vec<IpAddr>);
        assert_eq!(args.monitoring, false);
        assert_eq!(args.server, "https://monitoring.narrow.so");
        assert_eq!(args.key, "");
    }
}
