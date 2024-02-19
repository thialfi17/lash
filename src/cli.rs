use clap::{Parser, Subcommand};

use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
#[command(arg_required_else_help = true)]
pub struct Cli {
    /// Map "dot-" at start of directory names in source directory to "." in target names.
    #[arg(long)]
    pub dotfiles: bool,

    /// Do not change any files.
    #[arg(short = 'n', long)]
    pub dry_run: bool,

    /// Print more information about the files being processed.
    #[arg(short, long)]
    pub verbose: bool,

    /// Target directory to create links to package in. Defaults to parent of current directory
    #[arg(short, long)]
    pub target: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Install packages
    #[command(arg_required_else_help = true)]
    Link {
        #[arg(name = "PACKAGES")]
        packages: Vec<PathBuf>,
        #[arg(long)]
        adopt: bool,
    },

    /// Remove packages
    #[command(arg_required_else_help = true)]
    Unlink {
        #[arg(name = "PACKAGES")]
        packages: Vec<PathBuf>,
    },

    /// Remove and re-install packages
    #[command(arg_required_else_help = true)]
    Relink {
        #[arg(name = "PACKAGES")]
        packages: Vec<PathBuf>,
    },
}
