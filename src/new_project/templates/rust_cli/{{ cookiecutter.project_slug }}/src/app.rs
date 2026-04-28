use std::error::Error;

use crate::{
    cli::{Cli, Commands},
    config::{config_path, Config},
};

pub fn run(cli: Cli) -> Result<(), Box<dyn Error>> {
    let config = Config::load()?;

    match cli.command {
        Commands::Greet { name, count } => greet(name, count, config),
        Commands::ConfigPath => show_config_path(),
    }

    Ok(())
}

fn greet(name: Option<String>, count: Option<u8>, config: Config) {
    let name = name
        .or(config.name)
        .unwrap_or_else(|| "World".to_owned());
    let count = count.or(config.count).unwrap_or(1);

    for _ in 0..count {
        println!("Hello, {name}!");
    }
}

fn show_config_path() {
    match config_path() {
        Some(path) => println!("{}", path.display()),
        None => println!("No home directory was detected on this system."),
    }
}
