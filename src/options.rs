use std::borrow::Borrow;
use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;

use crate::cli::Cli;
use crate::config::Config;

#[derive(Debug)]
pub enum Command {
    /// Install packages
    Link,
    /// Remove packages
    Unlink,
}

#[derive(Debug)]
/// Struct containing the finalised options for the command.
///
/// All of the options have to be filled in by this point.
pub struct Options {
    /// Map "dot-" at start of directory names in source directory to "." in target names.
    pub dotfiles: bool,
    /// Do not change any files.
    pub dry_run: bool,
    /// Print more information about the files being processed.
    pub verbose: bool,
    /// Target directory to create links to package in. Defaults to parent of current directory
    pub target: PathBuf,
    /// The selected command
    pub command: Command,
    /// "Adopt" files already existing on the file system into the package. This is done by
    /// replacing the source file with the existing file. The link is still created as normal.
    pub adopt: bool,
    /// List of packages to install/remove
    pub packages: Vec<PathBuf>,
}

impl Options {
    /// Parse the CLI flags and attempt to read the configuration files.
    ///
    /// Merges the outputs to generate the options that the command should be ran with.
    pub fn new() -> Result<Self> {
        let cli_options = Cli::parse();
        let config_options = Config::new()?;

        Self::merge(cli_options.borrow(), config_options.borrow())
    }

    /// Merge the options from the command line and the configuration files. All of the potential
    /// options need to have a value.
    fn merge(cli: &Cli, config: &Config) -> Result<Self> {
        let dotfiles = config.dotfiles.unwrap_or(false) | cli.dotfiles;
        let verbose = config.verbose.unwrap_or(false) | cli.verbose;
        // TODO: Why does the options enum *have* to contain a value for adopt when it's only used
        // for some operations?
        let adopt = config.adopt.unwrap_or(false)
            | match cli.command {
                crate::cli::Command::Link { adopt, .. } => adopt,
                crate::cli::Command::Unlink { .. } => false,
            };

        let mut raw_target = cli.target.to_owned().or(config.target.to_owned());
        let raw_target =
            raw_target.get_or_insert(dirs::home_dir().expect("Could not get home dir!"));
        let target = shellexpand::full(
            raw_target
                .to_str()
                .expect("Target couldn't be converted to a str. Is it UTF-8?"),
        )?;

        // Can use unwrap_unchecked here for these values because we know they have to
        // have been set when using the ::from on a `Cli` value.
        Ok(Self {
            dotfiles,
            dry_run: cli.dry_run,
            verbose,
            target: target.into_owned().into(),
            command: match cli.command {
                crate::cli::Command::Link { .. } => Command::Link,
                crate::cli::Command::Unlink { .. } => Command::Unlink,
            },
            adopt,
            packages: match &cli.command {
                crate::cli::Command::Link { packages, .. } => packages.to_owned(),
                crate::cli::Command::Unlink { packages } => packages.to_owned(),
            },
        })
    }
}
