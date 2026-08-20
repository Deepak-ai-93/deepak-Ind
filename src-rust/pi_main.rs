#![allow(dead_code, unused_imports)]

mod agent;
mod benchmark;
mod budget;
mod config;
mod context;
mod memory;
mod policy;
mod providers;
mod repl;
mod routing;
mod security;
mod tasks;
mod tools;
mod usage;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = config::load_config(None);
    repl::start_repl(cfg).await
}
