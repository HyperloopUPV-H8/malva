use clap::Parser;
use std::{
    io,
    path::{Path, PathBuf},
    process::{exit, Command, Output},
};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
/// Easily create a new project from a template
///
/// This tool will copy a template project, changing the project name (the destination folder name by default)
struct TPCP {
    /// Destination folder, must not exist
    destination: PathBuf,
}

fn main() {
    let args = TPCP::parse();

    if args.destination.exists() {
        eprintln!("destination folder already exists");
        exit(1);
    } 

    handle_output(
        copy_dir("/opt/hyperfw/template-project", args.destination.as_path()),
        "failed to copy files from template project",
    );

    let project_name = args
            .destination
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();

    handle_output(
        find_and_replace(
            "template-project",
            project_name.as_str(),
            args.destination.join("CMakeLists.txt").as_path(),
        ),
        "failed to update CMakeLists.txt",
    );

    handle_output(
        find_and_replace(
            "template-project",
            project_name.as_str(),
            args.destination.join("build").join("build.sh").as_path(),
        ),
        "failed to update build.sh",
    );
}

fn copy_dir(src: &str, dst: &Path) -> io::Result<Output> {
    Command::new("cp").arg("-r").arg(src).arg(dst).output()
}

fn find_and_replace(previous: &str, new: &str, path: &Path) -> io::Result<Output> {
    Command::new("sed")
        .arg("-i")
        .arg(format!(
            "s/{previous}/{new}/g",
            previous = previous,
            new = new
        ))
        .arg(path)
        .output()
}

fn handle_output(output: io::Result<Output>, error_message: &str) {
    match output {
        Ok(output) if !output.status.success() => {
            eprintln!("{} ({})", error_message, output.status);
            exit(1);
        }
        Err(err) => {
            eprintln!("{} ({})", error_message, err);
            exit(1);
        }
        _ => (),
    }
}
