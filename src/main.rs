//! # Lash
//!
//! Lash is a commandline application that is intended to replace GNU Stow. It is not intended to
//! be a drop-in replacement but it does aim to replace most of the implemented functionality of
//! GNU Stow. To see the key differences refer to [GNU Stow Differences](crate#compared-to-gnu-stow)
//!
//! # Glossary
//!
//! - _package_: A directory containing a file structure that you wish to install or manage
//!   elsewhere on the filesystem - potentially mixed in with other files e.g. in your config
//!   directory or `/usr/bin`
//!
//! - _target directory_: The base directory to install the files from the package in. The
//!   structure inside the package will be mirrored in the _target_ directory and symbolic links will
//!   be made to connect the files.
//!
//! # Configuration
//!
//! Lash can be configured in several different ways. Global defaults can be configured with a
//! configuration file in the user's configuration directory. This respects the XDG User Dirs
//! specification. For workarea specifc configuration options a config file can be created in each
//! workarea.
//!
//! Configuration files should be called `lash.toml` and are (predictably) in the TOML format.
//! Options are specified in the global namespace. No package specific options can be configured.
//! To see the supported options in the configuration file see [Config](crate::config::Config)
//!
//! Most options can also be specified on the commandline.
//!
//! # Compared to GNU Stow
//!
//! - Configured by TOML files called `lash.toml`
//! - Commandline arguments do not match (`lash link` vs `stow -S`)
//! - Does not fold any of the directory structure (lash always creates the folders and links
//!   invididual files rather than attempting to minimize the number of links created).
//! - The `--dotfiles` option has been fixed. None of the bugs that plague GNU Stow are a problem
//!   in this implementation. I am aware some fixes had been made and are available in patches but
//!   even then some bugs remained (try using `--dotfiles` and `--adopt` with GNU Stow and the
//!   patches!).

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[allow(unused_imports)]
use log::{debug, error, info, warn};

use anyhow::Result;
use simplelog::{ColorChoice, LevelFilter, TermLogger, TerminalMode};

use crate::options::Options;

mod cli;
mod command;
mod config;
mod link;
mod options;

fn main() -> Result<()> {
    let options = Options::new()?;

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

    debug!("{:?}", options);

    let data_dir: PathBuf = [
        #[cfg(debug_assertions)]
        PathBuf::from("./"),
        #[cfg(not(debug_assertions))]
        dirs::data_dir().unwrap_or_else(|| PathBuf::from("~/.local/share/lash/")),
        #[cfg(not(debug_assertions))]
        "lash".into(),
        "store.bin".into(),
    ]
    .iter()
    .collect();

    debug!("Loading store from {:?}", data_dir);

    let mut store: HashMap<PathBuf, PathBuf> = {
        if !data_dir.exists() {
            debug!("Store does not exist, continuing");
            if let Err(e) = std::fs::create_dir_all(data_dir.parent().unwrap()) {
                error!(
                    "Failed to create directory for store: {:?}",
                    data_dir.parent()
                );
                return Err(e.into());
            }
            HashMap::new()
        } else {
            let mut file = match fs::File::open(&data_dir) {
                Ok(file) => file,
                Err(e) => {
                    error!("Failed to read data from the store: {:?}", e);
                    return Err(e.into());
                }
            };

            let config = bincode::config::standard();
            let store: HashMap<PathBuf, PathBuf> =
                match bincode::decode_from_std_read(&mut file, config) {
                    Ok(data) => data,
                    Err(e) => {
                        return Err(e.into());
                    }
                };
            debug!("Store loaded");
            store
        }
    };

    debug!("Store contents: {:?}", store);

    for res in command::process_packages(&options, &mut store) {
        match res {
            Ok(p) => info!("Successfully processed package {:?}", p),
            Err((p, e)) => warn!("Failed to process package {:?} due to: {}", p, e),
        }
    }

    {
        let mut file = match fs::File::create(&data_dir) {
            Ok(file) => file,
            Err(e) => {
                error!("Failed to open the store to write data: {:?}", e);
                return Err(e.into());
            }
        };
        let config = bincode::config::standard();
        bincode::encode_into_std_write(&store, &mut file, config)?;
    }

    Ok(())
}
