mod app;
mod cli;
mod config;
mod git;
mod models;
mod ssh;
mod ui;

use clap::Parser;
use cli::Cli;
use std::io;

fn main() -> io::Result<()> {
    let cli = Cli::parse();
    app::run(cli)
}
