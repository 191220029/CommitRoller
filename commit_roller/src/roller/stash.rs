use std::{path::{PathBuf, Path}, fs::{File, create_dir_all, remove_dir_all}, io::{BufReader, Write}, process::Command};

use crate::command::command_output::command_output;

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
        let (repo_new, repo_old, repo_root) = match copy_repo(repo_dir, out_dir, context){
            Ok((repo_new, repo_old, repo_root)) => (repo_new, repo_old, repo_root),
            Err(err) => {
                eprintln!("Fail to make copy of repo to {:?}\n{}", context, err);
                return;
            }
        };

        // println!("{:?}\n{:?}", repo_new, repo_old);

        match checkout(&repo_new, &context.hash_cur) {
            Ok(_) => 
                eprintln!("succesfully stash {:?} to commit {:?}",  context.title, context.hash_cur),
            Err(err) => 
                eprintln!("Fail to stash {:?} to commit {:?}\n{}", context.title, context.hash_cur, err),
        }

        match build(&repo_new) {
            Ok(()) => 
                println!("succesfully build and install {:?} {:?}", &context.title, &context.hash_cur),
            Err(err) =>
                eprintln!("Fail to build {:?} {:?}\n{}", &context.title, &context.hash_cur, err),
        }

        match checkout(&repo_old, &context.hash_old) {
            Ok(_) => 
                eprintln!("succesfully stash {:?} to commit {:?}",  context.title, context.hash_old),
            Err(err) => 
                eprintln!("Fail to stash {:?} to commit {:?}\n{}", context.title, context.hash_old, err),
        }
    
        match build(&repo_old) {
            Ok(()) => 
                println!("succesfully build and install {:?} {:?}", &context.title, &context.hash_old),
            Err(err) =>
                eprintln!("Fail to build {:?} {:?}\n{}", &context.title, &context.hash_old, err),
        }

        match remove_dir_all(repo_root.as_path()){
            Ok(_) => (),
            Err(err) => 
                eprintln!("Fail to remove tmp dir {:?}\n{}", repo_root, err),
        }
    });
}

fn parse_commit_context_json(commit_context_json: &PathBuf) -> anyhow::Result<Vec<LogContext>> {
    let fptr = File::open(commit_context_json)?;
    let reader = BufReader::new(fptr);

    let contexts = serde_json::from_reader(reader)?;
    Ok(contexts)
}

fn copy_repo(repo_dir: &PathBuf, out_dir: &PathBuf, context: &LogContext) -> anyhow::Result<(PathBuf, PathBuf, PathBuf)> {
    let out_dir = out_dir.join(&context.title.replace(' ', "_"));

    create_dir_all(&out_dir)?;

    let new_repo = out_dir.join(context.hash_cur.clone() + "_cur");
    let old_repo = out_dir.join(context.hash_old.clone() + "_old");

    if new_repo.is_dir() {
        eprintln!("warning: {:?} already exists.", &new_repo);
    } else {
        copy(&repo_dir, &new_repo)?;
            // .with_context(||{format!("Fail to copy repo from {:?} to {:?}", repo_dir, new_repo)})?;
        write_config_and_create_target_dir(&new_repo, &PathBuf::from(&context.title.replace(' ', "_")).join(context.hash_cur.clone() + "_cur"))?;
        println!("succesfully create copy of repo: {:?} -> {:?}", repo_dir, new_repo);
    }

    if old_repo.is_dir() {
        eprintln!("warning: {:?} already exists.", &old_repo);
    } else {
        copy(&repo_dir, &old_repo)?;
            // .with_context(||{format!("Fail to copy repo from {:?} to {:?}", repo_dir, new_repo)})?;
            write_config_and_create_target_dir(&old_repo, &PathBuf::from(&context.title.replace(' ', "_")).join(context.hash_old.clone() + "_old"))?;
        println!("succesfully create copy of repo: {:?} -> {:?}", repo_dir, old_repo);
    }

    Ok((new_repo, old_repo, out_dir))
}

#[cfg(unix)]
fn copy(from: &Path, to: &Path) -> anyhow::Result<()> {
    let mut cmd = Command::new("cp");
    cmd.arg("-pLR").arg(from).arg(to);
    command_output(&mut cmd)?;
    Ok(())
}

fn checkout(dir: &PathBuf, commit_id: &String) -> anyhow::Result<()> {
    let mut cmd = Command::new("git");
    cmd.current_dir(dir)
        .arg("checkout")
        .arg(commit_id);

    let _ = command_output(&mut cmd)?;

    // cmd = Command::new("git");
    // cmd.current_dir(dir)
    //     .arg("add")
    //     .arg(".");
    
    // let _ = command_output(&mut cmd)?;

    // cmd = Command::new("git");
    // cmd.current_dir(dir)
    //     .arg("commit")
    //     .arg("-m")
    //     .arg(commit_id);
    
    let _ = command_output(&mut cmd)?;

    // println!("{:?}", output);

    Ok(())
}

fn write_config_and_create_target_dir(repo_dir: &PathBuf, target_dir: &PathBuf) -> anyhow::Result<()> {
    let mut fptr = File::create(repo_dir.join("config.toml"))?;
    
    let content = "[build]\nbuild = \"x86_64-unknown-linux-gnu\"\n# build-dir = \"/media/workstation/device/home/fxl/rustc/baseline\"\n# cargo = \"/home/workstation/.rustup/toolchains/1.43-x86_64-unknown-linux-gnu/bin/cargo\"\n# rustc = \"/home/workstation/.rustup/toolchains/1.43-x86_64-unknown-linux-gnu/bin/rustc\"\ntarget = [\"x86_64-unknown-linux-gnu\"]\n[install]\n";
    let prefix = String::from("/media/workstation/device/home/fxl/rustc/targets/") + target_dir.to_str().unwrap();
    let sysconfdir = "sysconfdir = \"./etc\"\n";

    fptr.write(content.as_bytes())?;
    fptr.write("prefix = \"".as_bytes())?;
    fptr.write(prefix.as_bytes())?;
    fptr.write("\"\n".as_bytes())?;
    fptr.write(sysconfdir.as_bytes())?;

    create_dir_all(prefix)?;

    Ok(())
}

fn build(dir: &PathBuf) -> anyhow::Result<()> {
    let mut cmd = Command::new("./x.py");
    cmd.current_dir(dir)
        .arg("build");
    
    let _ = command_output(&mut cmd)?;
    
    let mut cmd = Command::new("./x.py");
    cmd.current_dir(dir)
        .arg("install");
    
    let _ = command_output(&mut cmd)?;

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