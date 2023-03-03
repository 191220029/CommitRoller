use std::{path::PathBuf, process::Command, fs::File, io::{BufReader, BufRead, Write, BufWriter}};

use chrono::{DateTime, Datelike, Duration, FixedOffset, Utc, TimeZone, Local};
use serde::{Serialize, Deserialize};

use crate::command::command_output::run_command_with_output;

#[derive(Deserialize, Debug)]
struct Gitlog {
    // hash: Option<String>,
    title: String,
    commits: Vec<String>,
    date: String,
    start_date: Option<String>
}

#[derive(Serialize, Debug)]
pub struct LogContext{
    pub hash_cur: String,
    pub hash_old: String,
    pub title: String
}



pub fn find_commits(repo_dir: &PathBuf, commits_json: &PathBuf, out: &PathBuf) {
    let mut contexts = vec![];

    let logs = match parse_commit_json(commits_json) {
        Ok(logs) => logs,
        Err(err) => {
            eprintln!("Fail to parse commit json file.\n{}", err);
            return;
        }
    };

    // let log_contexts = vec![];

    let date_fmt = "%Y-%m-%d %H:%M:%S";

    for log in logs {
        let date = log.date + " 00:00:00";
        match get_context_log(
            repo_dir, 
            &log.title, 
            &log.commits,
            match Local.datetime_from_str(&date, date_fmt) {
                Ok(date) => date,
                Err(err) => {
                    eprintln!("Fail to parse date string {}\n{}", &date, err);
                    continue;
                }
            },
            match log.start_date {
                Some(start_date) => {
                    let start_date = start_date + " 00:00:00";
                    match Local.datetime_from_str(&start_date, date_fmt) {
                        Ok(date) => Some(date),
                        Err(err) => {
                            eprintln!("Fail to parse start_date string {}\n{}", &date, err);
                            continue;
                        }
                    }
                }
                None => None,
            }
        ) {
            Ok(context) => {
                contexts.push(context);
            },
            Err(err) => {
                eprintln!("Fail to find context commit log for {}\n{}", log.title, err);                    
                continue;
            }
        }
    }

    match write_context(out, contexts){
        Ok(()) => (),
        Err(err) => {
            eprintln!("Fail to write commit contexts to file {:?}\n{}", out, err);
        }
    }
}

fn parse_commit_json(commits_json: &PathBuf) -> anyhow::Result<Vec<Gitlog>> {
    let fptr = File::open(commits_json)?;
    let reader = BufReader::new(fptr);

    let logs = serde_json::from_reader(reader)?;
    Ok(logs)
}

fn get_context_log(repo_dir: &PathBuf, title: &String, commit_titles: &Vec<String>, date: DateTime<Local>, start_date: Option<DateTime<Local>>) -> anyhow::Result<LogContext> {
    let last_day;
    match start_date {
        Some(start_date) => last_day = start_date - Duration::days(1),
        None => last_day = date - Duration::days(1)
    }
    let next_day = date + Duration::days(1);

    let mut cmd = Command::new("git");
    cmd.current_dir(repo_dir)
        .arg("log")
        .arg("--oneline")
        .arg("--before")
        .arg(format!("{}-{}-{}", date.year(), date.month(), date.day()))
        .arg("--after")
        .arg(format!("{}-{}-{}", last_day.year(), last_day.month(), last_day.day()));
        
    // println!("searching {}, commits{:?}, cmd = {:?}", title, commit_titles, &cmd);

    let output = match run_command_with_output(&mut cmd){
        Ok(output) => output,
        Err(err) => return Err(err),
    };

    let stdout = String::from_utf8(output.stdout.clone()).expect("utf8 output");

    let mut iter = stdout.lines().into_iter();
    let mut line = iter.next();

    let old_commit_title = commit_titles.first().unwrap();
    let cur_commit_title = commit_titles.last().unwrap();

    let mut old_commit_hash = "";
    let mut cur_commit_hash = "";

    while line != None {
        let s = line.unwrap();

        if s.contains(cur_commit_title) {
            cur_commit_hash = &s[0..s.find(' ').unwrap()];
        }
        if s.contains(old_commit_title) {
            line = iter.next();
            let s = line.unwrap();
            old_commit_hash = &s[0..s.find(' ').unwrap()];
            continue;
        }
        line = iter.next();   
    };

    assert!(cur_commit_hash.len() > 0);
    assert!(old_commit_hash.len() > 0);

    Ok(LogContext { 
        hash_cur: String::from(cur_commit_hash), 
        hash_old: String::from(old_commit_hash),
        title: title.clone()
    })
}

fn write_context(out: &PathBuf, contexts: Vec<LogContext>) -> anyhow::Result<()> {
    let fptr = File::create(out)?;
    let mut writer = BufWriter::new(fptr);

    writer.write(serde_json::to_string(&contexts)?.as_bytes())?;

    Ok(())
}

#[test]
fn test() {
    find_commits(
        &PathBuf::from("/media/workstation/device/home/fxl/rustc/rust"),
        &PathBuf::from("/media/workstation/device/home/fxl/CommitRoller/commit_roller/commit_info.json"),
        &PathBuf::from("/media/workstation/device/home/fxl/CommitRoller/commit_roller/out")
    )
}