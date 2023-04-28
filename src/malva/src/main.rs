use std::ffi::OsString;
use std::fmt::format;
use std::io::Stdout;
use std::path::PathBuf;
use std::path::Path;
use std::process::Child;
use std::process::{ Stdio, Output};
use std::io;
use clap::{arg, Command};
use std::process;

fn cli() -> Command {
    Command::new("malva")
        .about("Development framework for stm32 firmware development on vscode")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .allow_external_subcommands(true)
        .subcommand(
            Command::new("new")
                .about("Create a new firmware repository")
                .arg(arg!(<REMOTE> "Project name"))
                .arg_required_else_help(true),
        )
        .subcommand(
            Command::new("build")
                .about("Build the project into a binary/elf file")
                .arg(arg!(<PATH> "Path to CMakeLists.txt"))
               .arg(
                    arg!(-t --target [TARGET])
                        .value_parser(["nucleo", "board"])
                        .num_args(0..=1)
                        .default_value("nucleo")
                        .default_missing_value("board"),
                )
                .arg(
                    arg!(-p --profile <PROFILE>)
                        .value_parser(["debug", "release"])
                        .num_args(0..=1)
                        .default_value("debug")
                        .default_missing_value("release"),
                )
                .arg(
                    arg!(-e --noeth <NOETH>)
                        .value_parser(["true", "false"])
                        .num_args(0..=1)
                        .default_value("false")
                        .default_missing_value("true"),  
                ),
        )
        .subcommand(
            Command::new("flash")
                .about("flash binary to target")
        )
        .subcommand(
            Command::new("debug")
                .about("debug binary on target")
        )
}

fn copy_dir(src: &str, dst: &Path) -> io::Result<Output> {
    process::Command::new("cp").arg("-r").arg(src).arg(dst).output()
}

fn find_and_replace(previous: &str, new: &str, path: &Path) -> io::Result<Child> {

    println!("Replacing {} with {} in {}", previous, new, path.to_str().unwrap());
    let grep_child = process::Command::new("grep") 
        .arg("-rl")                  
        .arg(previous)
        .arg(path)
        .stdout(Stdio::piped())       
        .spawn()                    
        .expect("Could not find template project");
    
    process::Command::new("xargs")
        .arg("sed")
        .arg("-i")
        .arg(format!("s/template-project/{new}/g"))
        .stdin(Stdio::from(grep_child.stdout.unwrap())) // Pipe through.
        .stdout(Stdio::piped())
        .spawn()
}

fn cmake(cmake_path: &str) -> io::Result<Output> {
    println!("Running cmake in {}", cmake_path);
    process::Command::new("cmake")
        .arg(format!("-DCMAKE_TOOLCHAIN_FILE=arm-none-eabid.cmake"))
        .arg("-S")
        .arg(cmake_path)
        .arg("-B")
        .arg(format!("{cmake_path}/build"))
        .output()
}

fn make(make_path: &str) -> io::Result<Output> {
    println!("Running make in {}", make_path);
    process::Command::new("make")
        .arg("clean")
        .arg("-j16")
        .arg("-C")
        .arg(format!("{make_path}/build"))
        .arg("all")
        .output()
}
// fn find_dir_and_change(dir: &str) -> io::Result<Output> {
//     let find_child = process::Command::new("find") 
//         .arg(".")
//         .arg("-type")                  
//         .arg("d")
//         .arg("-name")
//         .arg(dir)
//         .stdout(Stdio::piped())       
//         .spawn()                    
//         .unwrap();

//     process::Command::new("cd")
//         .arg(Stdio::from(find_child.stdout.unwrap())) // Pipe through.
//         .output()

fn main() {
    let matches = cli().get_matches();

    match matches.subcommand() {
        Some(("new", sub_matches)) => {
            let path_str = sub_matches.get_one::<String>("REMOTE").expect("required");
            let project_path = PathBuf::from(path_str);
            println!(
                "Cloning {}...",
                project_path.to_str().unwrap()
            );
            copy_dir("/opt/malva/template-project", project_path.as_path()).expect("Can't copy ");
        
            let project_name = project_path
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string();

            find_and_replace("template-project", &project_name, project_path.as_path()).expect("Could not replace template project");
            
        }
        Some(("build", sub_matches)) => {
            let cmake_path = sub_matches.get_one::<String>("PATH").expect("required");

            let target = sub_matches.try_get_one::<String>("TARGET");
            let profile = sub_matches.try_get_one::<String>("PROFILE");
            let noeth = sub_matches.try_get_one::<String>("NOETH");
            
            cmake(cmake_path.as_str()).expect("Could not run cmake");
            make(cmake_path.as_str()).expect("Could not run make");
            
        }
        Some(("push", sub_matches)) => {
            println!(
                "Pushing to {}",
                sub_matches.get_one::<String>("REMOTE").expect("required")
            );
        }
        Some(("add", sub_matches)) => {
            let paths = sub_matches
                .get_many::<PathBuf>("PATH")
                .into_iter()
                .flatten()
                .collect::<Vec<_>>();
            println!("Adding {paths:?}");
        }
        Some(("stash", sub_matches)) => {
            let stash_command = sub_matches.subcommand().unwrap_or(("push", sub_matches));
            match stash_command {
                ("apply", sub_matches) => {
                    let stash = sub_matches.get_one::<String>("STASH");
                    println!("Applying {stash:?}");
                }
                ("pop", sub_matches) => {
                    let stash = sub_matches.get_one::<String>("STASH");
                    println!("Popping {stash:?}");
                }
                ("push", sub_matches) => {
                    let message = sub_matches.get_one::<String>("message");
                    println!("Pushing {message:?}");
                }
                (name, _) => {
                    unreachable!("Unsupported subcommand `{}`", name)
                }
            }
        }
        Some((ext, sub_matches)) => {
            let args = sub_matches
                .get_many::<OsString>("")
                .into_iter()
                .flatten()
                .collect::<Vec<_>>();
            println!("Calling out to {ext:?} with {args:?}");
        }
        _ => unreachable!(), // If all subcommands are defined above, anything else is unreachable!()
    }

    // Continued program logic goes here...
}
