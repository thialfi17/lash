use clap::Parser;
use log::{debug, SetLoggerError};
use simplelog::{ColorChoice, Config, LevelFilter, TermLogger, TerminalMode};

use crate::cli::Cli;

mod cli;
mod command;
mod links;

fn main() -> Result<(), SetLoggerError> {
    let cli = Cli::parse();

    let log_level = match cli.verbose {
        true => LevelFilter::Debug,
        false => LevelFilter::Info,
    };

    let _ = TermLogger::init(
        log_level,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    );

    debug!("{:?}", cli);

    let target = cli.target.unwrap_or("~".into());

    match cli.command {
        cli::Command::Link {
            ref packages,
            adopt,
        } => {
            command::link(packages, &target, cli.dotfiles, adopt);
        }
        cli::Command::Unlink { ref packages } => {
            command::unlink(packages, &target, cli.dotfiles);
        }
        cli::Command::Relink { ref packages } => {
            command::unlink(packages, &target, cli.dotfiles);
            command::link(packages, &target, cli.dotfiles, false);
        }
    };

    Ok(())
}
