use clap::Parser;
use serde::{Deserialize, Serialize};
use trust_dns_server::client::rr::RecordType;

#[derive(Parser, Clone, Debug)]
#[command(author, version, about, long_about=None)]
pub struct Cli {
    /// Path of YAML config file
    #[clap(long, short)]
    pub config: String,
    /// Path of MaxMind DB country database file
    #[clap(long, short)]
    pub mmdb: String,
    /// Always use this IP address for geolocation
    #[clap(long, short)]
    pub use_ip: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeoRecord {
    /// Value
    pub value: String,
    /// Country geolocation
    pub country: Option<String>,
    /// Continent geolocation
    pub continent: Option<String>,
    /// Time to live, will use base default if not given
    pub ttl: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Record {
    /// Domain name
    pub name: String,
    /// Record type
    #[serde(rename = "type")]
    pub record_type: RecordType,
    /// Default value
    pub value: String,
    /// Geolocation routing records
    #[serde(default = "empty_vec")]
    pub geo: Vec<GeoRecord>,
    /// Time to live
    #[serde(default = "ttl_default")]
    pub ttl: u32,
    /// MX preference
    pub preference: Option<u16>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Config {
    /// UDP sockets to listen on.
    #[serde(default = "udp_default")]
    pub udp: Vec<String>,
    /// TCP sockets to listen on.
    #[serde(default = "tcp_default")]
    pub tcp: Vec<String>,
    /// Records
    pub records: Vec<Record>,
}

fn udp_default() -> Vec<String> {
    vec![String::from("0.0.0.0:53"), String::from("[::]:53")]
}

fn tcp_default() -> Vec<String> {
    vec![String::from("0.0.0.0:53"), String::from("[::]:53")]
}

fn ttl_default() -> u32 {
    86400
}

fn empty_vec<T>() -> Vec<T> {
    vec![]
}
