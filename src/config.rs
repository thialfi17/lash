use std::path::PathBuf;

use config::{ConfigError, File};
use dirs::config_dir;
use serde_derive::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub verbose: Option<bool>,
    pub dotfiles: Option<bool>,
    pub target: Option<PathBuf>,
    pub adopt: Option<bool>,
}

impl Config {
    pub fn new() -> Result<Self, ConfigError> {
        let mut global_conf: PathBuf = config_dir().expect("~/.config/");
        global_conf.push("lash");

        let mut builder =
            config::Config::builder().add_source(File::with_name("lash"));

        if let Some(path) = global_conf.to_str() {
            builder = builder.add_source(File::with_name(path).required(false));
        }

        let config = builder.build()?;

        config.try_deserialize()
    }
}
