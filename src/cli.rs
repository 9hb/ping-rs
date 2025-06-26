//
// DESCRIPTION: defines the command line interface
//
use clap::Parser;

#[derive(Parser)]
#[command(name = "ping-rs")]
#[command(about = "Simple ping utility written in Rust")]
pub struct Args {
    /// Target address or hostname to ping
    pub target_addr: String,

    /// Number of ping packets to send
    #[arg(short = 'c', long = "count", default_value = "4")]
    pub amount: u32,

    /// Timeout in seconds
    #[arg(short = 't', long = "timeout", default_value = "5")]
    pub timeout: u64,

    /// Data size in bytes
    #[arg(short = 's', long = "size", default_value = "32")]
    pub data_size: usize,

    /// Interval between pings in milliseconds
    #[arg(short = 'i', long = "interval", default_value = "1000")]
    pub interval: u64,
}
