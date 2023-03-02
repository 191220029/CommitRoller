use std::path::PathBuf;

use self::gitlog::LogContext;

pub mod gitlog;

fn create_roll_back(repo_dir: &PathBuf, out_dir: &PathBuf, contexts: Vec<LogContext>) {

}