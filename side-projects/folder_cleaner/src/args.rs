use std::path::PathBuf;

use clap::{Arg, Parser, Subcommand, command};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args
{
    #[command(subcommand)]
    pub action:       Action,
    ///Path to the folder which will be cleaned.
    pub target:       PathBuf,
    ///Regex to match files by name
    #[arg(long, short)]
    pub matching:     Option<String>,
    ///Exact string to match file extensions.
    #[arg(long, short)]
    pub extension:    Option<String>,
    ///Match files smaller than BYTES
    #[arg(long, short)]
    pub smaller_than: Option<usize>,
    ///Match files larger than BYTES
    #[arg(long, short)]
    pub larger_than:  Option<usize>,
}

#[derive(Subcommand, Debug)]
pub enum Action
{
    ///Move the targeted files to a specified folder, creating it if it doesn't exist, after confirming.
    Move
    {
        dst: PathBuf
    },
    ///Delete the selected files, with a confirmation message.
    Delete,
}
