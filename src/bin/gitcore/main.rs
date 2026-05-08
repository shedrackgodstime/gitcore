mod app;
mod cli;
mod ui;

use crate::cli::Cli;
use clap::Parser;
use std::io;

fn main() -> io::Result<()> {
    #[cfg(windows)]
    let _ = colored::control::set_virtual_terminal(true);

    let cli = Cli::parse();
    app::run(cli)
}
