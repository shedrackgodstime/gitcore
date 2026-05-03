mod app;
mod cli;
mod config;
mod git;
mod gpg;
mod models;
mod ssh;
mod ui;
mod vault;

use clap::Parser;
use cli::Cli;
use std::io;

fn main() -> io::Result<()> {
    let cli = Cli::parse();
    app::run(cli)
}
