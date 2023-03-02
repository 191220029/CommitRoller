use clap::Parser;
use roller::{gitlog::find_commits, stash::stash_all};

use crate::cli::Cli;

mod cli;
mod command;
mod roller;

fn main() {
    let args = Cli::parse();
    match args.subcommand {
        cli::Commands::FindCommit { 
            repo_dir, 
            commits_json, 
            out 
        } => {
            find_commits(&repo_dir, &commits_json, &out);
        },
        cli::Commands::RollBack { 
            repo_dir, 
            commit_context_json, 
            out_dir,
        } => {
            stash_all(&repo_dir, &out_dir, &commit_context_json);
        },
    }
}
