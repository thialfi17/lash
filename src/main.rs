use std::env::VarError;

use ::config::ConfigError;
use clap::Parser;
use log::debug;
use shellexpand::LookupError;
use simplelog::{ColorChoice, LevelFilter, TermLogger, TerminalMode};

use crate::cli::Cli;
use crate::options::{Command, Options};

mod cli;
mod config;
mod options;

mod command;
mod link;

enum Error {
    Config(ConfigError),
    Target(LookupError<VarError>),
}

impl From<ConfigError> for Error {
    fn from(value: ConfigError) -> Self {
        Self::Config(value)
    }
}
impl From<LookupError<VarError>> for Error {
    fn from(value: LookupError<VarError>) -> Self {
        Self::Target(value)
    }
}
impl core::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Config(arg0) => f.debug_tuple("Config").field(arg0).finish(),
            Self::Target(arg0) => f.debug_tuple("Target").field(arg0).finish(),
        }
    }
}

fn main() -> Result<(), Error> {
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
        unimplemented!();
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
