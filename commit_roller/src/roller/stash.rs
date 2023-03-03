use std::{path::{PathBuf, Path}, fs::{File, create_dir_all}, io::BufReader};

use anyhow::Context;

use super::gitlog::LogContext;

pub fn stash_all(repo_dir: &PathBuf, out_dir: &PathBuf, commit_context_json: &PathBuf) {
    let contexts = match parse_commit_context_json(commit_context_json) {
        Ok(contexts) => contexts,
        Err(err) => {
            eprintln!("Fail to parse commit_context_json file {:?}\n{}", commit_context_json, err);
            return;
        }
    };

    contexts.iter().for_each(|context| {
        let (repo_new, repo_old) = match copy_repo(repo_dir, out_dir, context){
            Ok((repo_new, repo_old)) => (repo_new, repo_old),
            Err(err) => {
                eprintln!("Fail to make copy of repo to {:?}\n{}", repo_dir, err);
                return;
            }
        };

        println!("{:?}\n{:?}", repo_new, repo_old);
    });
}

fn parse_commit_context_json(commit_context_json: &PathBuf) -> anyhow::Result<Vec<LogContext>> {
    let fptr = File::open(commit_context_json)?;
    let reader = BufReader::new(fptr);

    let contexts = serde_json::from_reader(reader)?;
    Ok(contexts)
}

fn copy_repo(repo_dir: &PathBuf, out_dir: &PathBuf, context: &LogContext) -> anyhow::Result<(PathBuf, PathBuf)> {
    let out_dir = out_dir.join(&context.title);

    create_dir_all(&out_dir)?;

    let new_repo = out_dir.join(context.hash_cur.clone() + "_cur");
    let old_repo = out_dir.join(context.hash_old.clone() + "_old");

    copy(&repo_dir, &new_repo)
        .with_context(||{format!("Fail to copy repo from {:?} to {:?}", repo_dir, new_repo)})?;

    copy(&repo_dir, &old_repo)
        .with_context(||{format!("Fail to copy repo from {:?} to {:?}", repo_dir, old_repo)})?;

    Ok((new_repo, old_repo))
}

#[cfg(unix)]
fn copy(from: &Path, to: &Path) -> anyhow::Result<()> {
    use std::process::Command;
    use crate::command::command_output::command_output;

    let mut cmd = Command::new("cp");
    cmd.arg("-pLR").arg(from).arg(to);
    command_output(&mut cmd)?;
    Ok(())
}

#[test]
fn test_stash() {
    stash_all(
        &PathBuf::from("/media/workstation/device/home/fxl/rustc/rust"),
        &PathBuf::from("/media/workstation/disk/fxl/rust"),
        &PathBuf::from("/media/workstation/device/home/fxl/CommitRoller/commit_roller/out/commit_context.json")
    )
}