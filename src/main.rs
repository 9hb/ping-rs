//
// DESCRIPTION: main entry point for this app
//
mod ping;
mod cli;
mod utils;

use clap::Parser;
use cli::Args;
use std::process;

#[tokio::main]
async fn main() {
    let args = Args::parse();

    if let Err(e) = ping::spustit_ping(args).await {
        eprintln!("✖ error: {}", e);
        process::exit(1);
    }
}
