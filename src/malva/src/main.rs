use std::path::{Path, PathBuf};
use std::fs::metadata;
use std::process::{self, exit};
use std::process::{Stdio, Output, Child};
use std::io;
use clap::{arg, Command, ArgMatches, Error};
use colored::Colorize;

fn cli() -> Command {
    Command::new("malva")
        .about("Development framework for stm32 firmware development on vscode")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .allow_external_subcommands(true)
        .subcommand(
            Command::new("new")
                .about("Create a new firmware repository")
                .arg(arg!(<NAME> "Project name"))
                .arg_required_else_help(true),
        )
        .subcommand(
            Command::new("build")
                .about("Build the project into a binary/elf file")
                .arg(
                    arg!(<PATH> "Path to CMakeLists.txt")
                    .default_value(".")
                )
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
                    arg!(-e --eth <ETH>)
                        .value_parser(["true", "false"])
                        .num_args(0..=1)
                        .require_equals(true)
                        .default_value("true")
                )
                .arg(
                    arg!(-f --flash <FLASH>)
                        .value_parser(["true", "false"])
                        .num_args(0..=1)
                        .require_equals(true)
                        .default_value("false")
                )                                                                                                     
        )
        .subcommand(
            Command::new("flash")
                .about("flash binary to target")
                .arg(
                    arg!(<BINARY> "Path to the .bin file")
                    .default_value(".")
                )
                .arg_required_else_help(true),
        )
        .subcommand(
            Command::new("update")
                .about("Update to latest version (changes ST-LIB and template-project)"))
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


fn new_command(sub_matches: &ArgMatches) {
    let path_str = sub_matches.get_one::<String>("NAME").expect("required");
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

fn build_command(sub_matches: &ArgMatches) {
    let cmake_path = sub_matches.get_one::<String>("PATH").expect("required");

    let target = sub_matches.get_one::<String>("target").map(|s| s.as_str()).expect("Defaulted in clap");
    let profile = sub_matches.get_one::<String>("profile").map(|s| s.as_str()).expect("Defaulted in clap");
    let eth = sub_matches.get_one::<String>("eth").map(|s| s.as_str()).expect("Defaulted in clap");
    let flash: &str = sub_matches.get_one::<String>("flash").map(|s | s.as_str()).expect("Defaulted in clap");
    
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


    let make_exit_status = process::Command::new("make")
        .arg("-j16")
        .arg("-C")
        .arg(format!("{cmake_path}/build"))
        .arg("all")
        .spawn()
        .expect("Could not run make. Is it installed?")
        .wait()
        .unwrap();

    let project_name = Path::new(cmake_path)
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();


    if !cmake_exit_status.success() || !make_exit_status.success() {
        println!("\n\n{}", "Build failed!".red().bold());
        exit(1)
    }

    let objcopy_exit_status = process::Command::new("arm-none-eabi-objcopy")
        .arg("-O")
        .arg("binary")
        .arg(format!("{cmake_path}/build/{project_name}.elf"))
        .arg(format!("{cmake_path}/build/{project_name}.bin"))
        .spawn()
        .expect("Could not run objcopy. Is it installed?")
        .wait()
        .unwrap();

    println!("\n\n{}", "Build succeeded!".green().bold());
    println!("\ntarget: {}\nprofile: {}\neth:{}", target, profile, eth);
    process::Command::new("mv")
        .arg(format!("{cmake_path}/build/compile_commands.json"))
        .arg(format!("{cmake_path}/compile_commands.json"))
        .spawn()
        .expect("Could not use mv. Is it installed?")
        .wait()
        .unwrap();


    if flash == "true" {
        flash_command(sub_matches, format!("{cmake_path}/build/{project_name}.bin").as_str());
    }
}

fn check_path(path: &str) -> &Path {
    match metadata(path) {
        Ok(res) => Path::new(path),
        Err(err) => {
            println!("{}: {}", "Path does not exist".red().bold(), path);
            exit(1)
        }
    }
}

fn get_file_or_dir(path: &Path) -> &str {
    match path.file_name() {
        Some(res) => to_str(path),
        None => {
            println!("{}", "Do not end file in ..".red().bold());
            exit(1)
        }
    }
}

fn to_str(path: &Path) -> &str {
    match path.to_str() {
        Some(res) => res,
        None => {
            println!("{}", "path exists but is not valid unicode".red().bold());
            exit(1)
        }
    }
}

fn flash_command(_sub_matches: &ArgMatches, path_str: &str) {
    let path = check_path(path_str);

    let binary_path;
    if path.is_dir() {
        let project_name = get_file_or_dir(path);
        let mut s = String::from(path_str);
        s.push_str("/build/");
        s.push_str(project_name);
        s.push_str(".bin");
        binary_path = check_path(s.as_str()).to_owned();
    } else {
        binary_path = path.to_path_buf();
    }

    let stflash_process = match process::Command::new("st-flash")
        .arg("write")
        .arg(binary_path)
        .arg("0x08000000")
        .spawn() {
            Ok(child) => child,
            Err(_err) => {
                println!("\n\nCould not run st-flash. Is it installed? (Check st-link open source tools on Github");
                exit(1);
            }
        }.wait();
    
    match stflash_process {
        Ok(res) => {
            if res.success() {
                println!("\n\n{}", "Flash succeeded!".green().bold());
            } else {
                println!("\n\n{}", "Flash failed!".red().bold());
            }
        },
        Err(err) => {
            println!("Could not run st-flash. Is it installed? (Check st-link open source tools on Github");
            exit(1)
        }
    }
}

fn main() {
    let matches = cli().get_matches();
    match matches.subcommand() {
        Some(("new", sub_matches)) => {
            new_command(sub_matches);
        }
        
        Some(("build", sub_matches)) => {
            build_command(sub_matches);
        }

        Some(("flash", sub_matches)) => {
            flash_command(sub_matches, sub_matches.get_one::<String>("BINARY").expect("required"));
        }

        _ => unreachable!(), // If all subcommands are defined above, anything else is unreachable!()
    }

    // Continued program logic goes here...
}
