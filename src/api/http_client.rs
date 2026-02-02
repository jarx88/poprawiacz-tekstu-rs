use once_cell::sync::Lazy;
use reqwest::Client;
use std::net::{IpAddr, Ipv4Addr};
use std::time::Duration;

use crate::error::{CONNECTION_TIMEOUT, DEFAULT_TIMEOUT};

pub static SHARED_CLIENT: Lazy<Client> = Lazy::new(|| {
    Client::builder()
        .timeout(Duration::from_secs(DEFAULT_TIMEOUT))
        .connect_timeout(Duration::from_secs(CONNECTION_TIMEOUT))
        .pool_max_idle_per_host(10)
        .pool_idle_timeout(Duration::from_secs(30))
        .tcp_keepalive(Duration::from_secs(30))
        .tcp_nodelay(true)
        // Force IPv4 to avoid IPv6 connection issues
        .local_address(IpAddr::V4(Ipv4Addr::UNSPECIFIED))
        .build()
        .expect("Failed to create HTTP client")
});

pub static STREAMING_CLIENT: Lazy<Client> = Lazy::new(|| {
    Client::builder()
        .connect_timeout(Duration::from_secs(CONNECTION_TIMEOUT))
        .read_timeout(Duration::from_secs(120))
        .pool_max_idle_per_host(10)
        .pool_idle_timeout(Duration::from_secs(90))
        .tcp_keepalive(Duration::from_secs(30))
        .tcp_nodelay(true)
        // Force IPv4 to avoid IPv6 connection issues
        .local_address(IpAddr::V4(Ipv4Addr::UNSPECIFIED))
        .build()
        .expect("Failed to create streaming HTTP client")
});

pub fn get_client() -> &'static Client {
    &SHARED_CLIENT
}

pub fn get_streaming_client() -> &'static Client {
    &STREAMING_CLIENT
}
