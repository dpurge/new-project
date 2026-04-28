use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "{{ cookiecutter.binary_name }}", version, about = "{{ cookiecutter.description }}")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Print a greeting, optionally using config values from the home directory.
    Greet {
        /// Name to greet.
        #[arg(long)]
        name: Option<String>,
        /// Number of greetings to print.
        #[arg(long)]
        count: Option<u8>,
    },
    /// Print the optional config file path.
    ConfigPath,
}
