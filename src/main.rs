use anyhow::Result;
use clap::Parser;
use log::debug;
use simplelog::{ColorChoice, LevelFilter, TermLogger, TerminalMode};

use crate::cli::Cli;
use crate::options::{Command, Options};

mod cli;
mod config;
mod options;

mod command;
mod link;

fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = crate::config::Config::new()?;
    let options = Options::new(&cli, &config)?;

    let log_level = match options.verbose {
        true => LevelFilter::Debug,
        false => LevelFilter::Info,
    };

    let _ = TermLogger::init(
        log_level,
        simplelog::Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    );

    if options.adopt {
        anyhow::bail!("--adopt hasn't been implemented yet!");
    }

    debug!("{:?}", options);

    match options.command {
        Command::Link => {
            command::link(&options);
        }
        Command::Unlink => {
            command::unlink(&options);
        }
        Command::Relink => {
            command::unlink(&options);
            command::link(&options);
        }
    };

    Ok(())
}
