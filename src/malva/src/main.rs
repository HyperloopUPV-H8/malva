use std::ffi::OsString;
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
                    arg!(-t --target <TARGET>)
                        .value_parser(["nucleo", "board"])
                        .num_args(0..=1)
                        .require_equals(true)
                        .default_value("nucleo")
                )
                .arg(
                    arg!(-p --profile <PROFILE>)
                        .value_parser(["debug", "release"])
                        .num_args(0..=1)
                        .require_equals(true)
                        .default_value("debug")
                )
                .arg(
                    arg!(-e --eth <NOETH>)
                        .value_parser(["true", "false"])
                        .num_args(0..=1)
                        .require_equals(true)
                        .default_value("true")
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

            let target = sub_matches.get_one::<String>("target").map(|s| s.as_str()).expect("Defaulted in clap");
            let profile = sub_matches.get_one::<String>("profile").map(|s| s.as_str()).expect("Defaulted in clap");
            let eth = sub_matches.get_one::<String>("eth").map(|s| s.as_str()).expect("Defaulted in clap");
            
            let t: &str;
            if target == "nucleo" {
                t = "NUCLEO";
            } else {
                t = "BOARD";
            }

            let p: &str;
            if profile == "debug" {
                p = 
                "-g0";
            } else {
                p = 
                "-g3";
            }

            let e: &str;
            if eth == "true" {
                e = "HAL_ETH_MODULE_ENABLED";
            } else {
                e = "OFF";
            }


            let cmake_exit_status = process::Command::new("cmake")
                .arg(format!("-DCMAKE_TOOLCHAIN_FILE=arm-none-eabi.cmake"))
                .arg("-S")
                .arg(cmake_path)
                .arg("-B")
                .arg(format!("{cmake_path}/build"))
                .arg(format!("-D{t}=ON"))
                .arg(format!("-DPROFILE={p}"))
                .arg(format!("-D{e}=ON"))
                .spawn()
                .expect("Could not run cmake. Is it installed?")
                .wait()
                .unwrap();


            let make_exit_status =process::Command::new("make")
                .arg("-j16")
                .arg("-C")
                .arg(format!("{cmake_path}/build"))
                .arg("all")
                .spawn()
                .expect("Could not run make. Is it installed?")
                .wait()
                .unwrap();


            if cmake_exit_status.success() && make_exit_status.success() {
                println!("\n\nBuild successful!");
                println!("\ntarget: {}\nprofile: {}\neth:{}", target, profile, eth);
            } else {
                println!("Build failed!");
            }
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
