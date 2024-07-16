use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Optionally specify the path to the config file to use.
    /// If not set, then then the `TVTRACK_CONFIG_FILE` environment variable will be used.
    #[arg(short, long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    AddByTitle {
        title: String,
        first_air_year: Option<i32>,
    },
    AddById {
        tmdb_id: i32,
    },
    AddFrom {
        file_path: PathBuf,
    },
    Update {
        tmdb_id: Option<i32>,

        #[arg(short, long)]
        force: Option<bool>,
    },
}
