use anyhow::Result;
use clap::Parser;
use handler::Handler;
use options::{Cli, Config};
use std::fs::File;
use std::time::Duration;
use tokio::net::{TcpListener, UdpSocket};
use trust_dns_server::ServerFuture;

mod handler;
mod options;
mod util;

// Timeout for TCP connections.
const TCP_TIMEOUT: Duration = Duration::from_secs(10);

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();
    let reader = File::open(&cli.config)?;
    let config: Config = serde_yaml::from_reader(reader)?;
    let handler = Handler::new(&cli, &config);

    // create DNS server
    let mut server = ServerFuture::new(handler);

    // register UDP listeners
    for udp in &config.udp {
        server.register_socket(UdpSocket::bind(udp).await?);
    }

    // register TCP listeners
    for tcp in &config.tcp {
        server.register_listener(TcpListener::bind(&tcp).await?, TCP_TIMEOUT);
    }

    // run DNS server
    server.block_until_done().await?;

    Ok(())
}
