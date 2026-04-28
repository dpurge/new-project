mod app;
mod cli;
mod config;

use clap::Parser;

use crate::cli::Cli;

fn main() {
    let cli = Cli::parse();

    if let Err(error) = app::run(cli) {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}
