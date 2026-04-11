mod cli;
mod config;
mod tech;
mod logger;
mod util;
mod batch;

mod context;

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    println!("Hello, world!");

    Ok(())
}
