use clap::Parser;
use gitlog::find_commits;

use crate::cli::Cli;

mod gitlog;
mod cli;
mod command;

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
            commit_id_json, 
            out_dir 
        } => {
            
        },
    }
}
