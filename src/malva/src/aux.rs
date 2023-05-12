use std::fmt::format;
use std::time::Instant;
use std::path::{Path, PathBuf};
use std::fs::metadata;
use std::process::{self, exit, ExitStatus};
use std::process::{Stdio, Output, Child};
use std::io;
use clap::{arg, Command, ArgMatches, Error};
use colored::Colorize;

pub fn err_println(msg: &str) {
    eprintln!("{} {}", "Error:".red().bold(), msg);
    exit(1)
}

pub fn copy_dir(src: &str, dst: &Path) -> io::Result<Output> {
    process::Command::new("cp").arg("-r").arg(src).arg(dst).output()
}

pub fn find_and_replace(previous: &str, new: &str, path: &Path) -> () {

    println!("Replacing {} with {} in {}", previous, new, path.to_str().unwrap());
    let mut grep_command = process::Command::new("grep");

    let mut grep_child = match grep_command
        .arg("-rl")                  
        .arg(previous)
        .arg(path)
        .stdout(Stdio::piped()).spawn() {      
        Ok(res) => res,
        Err(err) => {
            eprintln!("{} {}", "Error:".red().bold(), err);
            exit(1)
        }
    };

    match grep_child.wait() {
        Ok(res) => res,
        Err(err) => {
            eprintln!("{} {}", "Error:".red().bold(), err);
            exit(1)
        }
    };
    
    let mut xargs_command = process::Command::new("xargs");

    let mut xargs_child = match xargs_command
        .arg("sed")
        .arg("-i")
        .arg(format!("s/template-project/{new}/g"))
        .stdin(Stdio::from(grep_child.stdout.unwrap())) // Pipe through.
        .stdout(Stdio::piped())
        .spawn() {

        Ok(res) => res,
        Err(err) => {
            eprintln!("{} {}", "Error:".red().bold(), err);
            exit(1)
        }
    };

    match xargs_child.wait() {
        Ok(_res) => (), 
        Err(err) => {
            eprintln!("{} {}", "Error:".red().bold(), err);
            exit(1)
        }
    }
}

pub fn get_match<'a>(sub_matches: &'a ArgMatches, name: &str) -> &'a String {
    match sub_matches.get_one::<String>(name) {
        Some(res) => res,
        None => {
            eprintln!("{} Wrong type for {} argument. See {}", "Error:".red().bold(), name.yellow().bold(), "malva build --help".yellow().bold());
            exit(1)
        }
    }
}

pub fn check_path(path: &str) -> &Path {
    match metadata(path) {
        Ok(res) => Path::new(path),
        Err(err) => {
            eprintln!("{}: {} path does not exist.", "Error: ".red().bold(), path);
            exit(1)
        }
    }
}

pub fn get_file_or_dir(path: &Path) -> &str {
    match path.file_name() {
        Some(res) => to_str(path),
        None => {
            eprintln!("{}Do not end file in ..", "Error: ".red().bold());
            exit(1)
        }
    }
}

pub fn to_str(path: &Path) -> &str {
    match path.to_str() {
        Some(res) => res,
        None => {
            println!("{}", "path exists but is not valid unicode".red().bold());
            exit(1)
        }
    }
}