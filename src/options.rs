use std::path::PathBuf;

#[derive(Debug)]
pub enum Command {
    Link,
    Unlink,
    Relink,
}

#[derive(Debug)]
pub struct Options {
    pub dotfiles: bool,
    pub dry_run: bool,
    pub verbose: bool,
    pub target: PathBuf,
    pub command: Command,
    pub adopt: bool,
    pub packages: Vec<PathBuf>,
}

#[derive(Debug)]
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
    pub fn new(cli: &crate::cli::Cli, config: &crate::config::Config) -> Self {
        let cli_options: MaybeOptions = cli.into();
        let config_options: MaybeOptions = config.into();

        Self::merge(cli_options, config_options)
    }

    fn merge(cli: MaybeOptions, config: MaybeOptions) -> Self {
        let dotfiles = config.dotfiles.unwrap_or(false) | cli.dotfiles.unwrap_or(false);
        let verbose = config.verbose.unwrap_or(false) | cli.verbose.unwrap_or(false);
        let adopt = config.adopt.unwrap_or(false) | cli.adopt.unwrap_or(false);

        unsafe {
            Self {
                dotfiles,
                dry_run: cli.dry_run.unwrap_unchecked(),
                verbose,
                target: cli
                    .target
                    .or(config.target)
                    .get_or_insert(dirs::home_dir().expect("Could not get home dir!"))
                    .to_path_buf(),
                command: cli.command.unwrap_unchecked(),
                adopt,
                packages: cli.packages.unwrap_unchecked(),
            }
        }
    }
}

impl From<&crate::config::Config> for MaybeOptions {
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

impl From<&crate::Cli> for MaybeOptions {
    fn from(value: &crate::Cli) -> Self {
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
