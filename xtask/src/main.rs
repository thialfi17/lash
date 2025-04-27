use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::{shells::Shell, Generator};

use std::fs::{self, File};
use std::io::Error;
use std::path::{Path, PathBuf};

#[derive(Parser, Debug)]
struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Generate the terminal completions for the script
    #[command(arg_required_else_help = true)]
    Completions {
        /// The shells to generate completions for
        #[arg(default_values_t = vec![Shell::Bash, Shell::Fish, Shell::Zsh])]
        shells: Vec<Shell>,

        /// The output directory to put the completion scripts
        #[arg(long = "target-dir")]
        target: PathBuf,
    },
}

fn gen_completions(out_dir: &Path, shells: &Vec<Shell>) -> Result<(), Error> {
    let mut cmd = lash::cli::Cli::command();

    fs::create_dir_all(out_dir)?;

    for &shell in shells {
        let file_name = shell.file_name("lash");
        let mut file = File::create(out_dir.join(file_name))?;
        clap_complete::generate(shell, &mut cmd, "lash", &mut file);
    }

    println!("Generated completions");

    Ok(())
}

fn main() -> Result<(), Error> {
    let cli = Cli::parse();

    match cli.command {
        Command::Completions { shells, target } => {
            gen_completions(&target, &shells)?;
        }
    }

    Ok(())
}
