use std::path::PathBuf;

use config::{ConfigError, File};
use dirs::config_dir;
use serde_derive::Deserialize;

/// This struct is what defines which options are supported in the TOML configuration files.
///
/// All of the options are optional.
#[derive(Debug, Deserialize)]
pub struct Config {
    pub verbose: Option<bool>,
    pub dotfiles: Option<bool>,
    pub target: Option<PathBuf>,
    pub adopt: Option<bool>,
}

impl Config {
    /// Attempts to read the configuration from the user's configuration directory or if not
    /// found/set `~/.config/`. Then attempts to read configuration from the current directory.
    pub fn new() -> Result<Self, ConfigError> {
        let mut global_conf: PathBuf = config_dir().expect("~/.config/");
        global_conf.push("lash");

        let mut builder =
            config::Config::builder().add_source(File::with_name("lash").required(false));

        if let Some(path) = global_conf.to_str() {
            builder = builder.add_source(File::with_name(path).required(false));
        }

        let config = builder.build()?;

        config.try_deserialize()
    }
}
