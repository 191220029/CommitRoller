use std::path::PathBuf;

#[derive(Debug, clap::Parser)]
#[clap(about, version, author)]
pub struct Cli {
    #[clap(subcommand)]
    pub subcommand: Commands
}

#[derive(Debug, clap::Subcommand)]
#[clap(rename_all = "snake_case")]
pub enum Commands {
    FindCommit {
        #[clap(long = "repo-dir")]
        repo_dir: PathBuf,

        #[clap(long = "commits-json")]
        commits_json: PathBuf,

        #[clap(long = "out")]
        out: PathBuf
    },
    RollBack {
        #[clap(long = "commit_id_json")]
        commit_id_json: PathBuf,

        #[clap(long = "out-dir")]
        out_dir: PathBuf
    }
}