use std::{env::VarError, path::PathBuf};

use shellexpand::LookupError;

#[derive(Debug)]
pub enum Command {
    /// Install packages
    Link,
    /// Remove packages
    Unlink,
    /// Remove and re-install packages
    Relink,
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

#[derive(Debug)]
/// Temporary struct used to map all of the options from different sources into the same structure.
/// This is then merged to create a [Options] struct with all of the values determined.
struct MaybeOptions {
    pub dotfiles: Option<bool>,
    pub dry_run: Option<bool>,
    pub verbose: Option<bool>,
    pub target: Option<PathBuf>,
    pub command: Option<Command>,
    pub adopt: Option<bool>,
    pub packages: Option<Vec<PathBuf>>,
}

impl Options {
    pub fn new(
        cli: &crate::cli::Cli,
        config: &crate::config::Config,
    ) -> Result<Self, LookupError<VarError>> {
        let cli_options: MaybeOptions = cli.into();
        let config_options: MaybeOptions = config.into();

        Self::merge(cli_options, config_options)
    }

    /// Merge the options from the command line and the configuration files. All of the potential
    /// options need to have a value.
    fn merge(cli: MaybeOptions, config: MaybeOptions) -> Result<Self, LookupError<VarError>> {
        let dotfiles = config.dotfiles.unwrap_or(false) | cli.dotfiles.unwrap_or(false);
        let verbose = config.verbose.unwrap_or(false) | cli.verbose.unwrap_or(false);
        let adopt = config.adopt.unwrap_or(false) | cli.adopt.unwrap_or(false);

        let mut raw_target = cli.target.or(config.target);
        let raw_target =
            raw_target.get_or_insert(dirs::home_dir().expect("Could not get home dir!"));
        let target = shellexpand::full(
            raw_target
                .to_str()
                .expect("Target couldn't be converted to a str. Is it UTF-8?"),
        )?;

        // Can use unwrap_unchecked here for these values because we know they have to
        // have been set when using the ::from on a `Cli` value.
        unsafe {
            Ok(Self {
                dotfiles,
                dry_run: cli.dry_run.unwrap_unchecked(),
                verbose,
                target: target.into_owned().into(),
                command: cli.command.unwrap_unchecked(),
                adopt,
                packages: cli.packages.unwrap_unchecked(),
            })
        }
    }
}

impl From<&crate::config::Config> for MaybeOptions {
    /// Some options are not settable in the config files such as dry_run, command and packages.
    /// These can never be `Some`.
    fn from(value: &crate::config::Config) -> Self {
        Self {
            dotfiles: value.dotfiles,
            dry_run: None,
            verbose: value.verbose,
            target: value.target.clone(),
            command: None,
            adopt: value.adopt,
            packages: None,
        }
    }
}

impl From<&crate::cli::Cli> for MaybeOptions {
    /// All options should be settable from the command line so all values should have the
    /// potential to be `Some`.
    fn from(value: &crate::cli::Cli) -> Self {
        let adopt = match value.command {
            crate::cli::Command::Link { adopt, .. } => Some(adopt),
            _ => None,
        };

        let packages = match value.command {
            crate::cli::Command::Link { ref packages, .. } => Some(packages.clone()),
            crate::cli::Command::Unlink { ref packages } => Some(packages.clone()),
            crate::cli::Command::Relink { ref packages } => Some(packages.clone()),
        };

        let command = match value.command {
            crate::cli::Command::Link { .. } => Command::Link,
            crate::cli::Command::Unlink { .. } => Command::Unlink,
            crate::cli::Command::Relink { .. } => Command::Relink,
        };

        Self {
            dotfiles: Some(value.dotfiles),
            dry_run: Some(value.dry_run),
            verbose: Some(value.verbose),
            target: value.target.clone(),
            command: Some(command),
            adopt,
            packages,
        }
    }
}
