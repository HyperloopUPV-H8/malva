mod aux;
use aux::{check_path, copy_dir, find_and_replace, get_file_or_dir, get_match, to_str, err_println};

use std::fmt::format;
use std::time::Instant;
use std::path::{Path, PathBuf};
use std::fs::metadata;
use std::process::{self, exit};
use std::process::{Stdio, Output, Child};
use std::io;
use std::env::current_dir;
use clap::{arg, Command, ArgMatches, Error};
use colored::Colorize;


fn cli() -> Command {
    Command::new("malva")
        .about(format!("\n{} - Development framework for stm32 firmware development on vscode", "malva v0.6.0".yellow().bold()))
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

fn run_command(command: &mut process::Command) -> Result<std::process::ExitStatus, std::io::Error> {
    match command.spawn() {
        Ok(child) => child,
        Err(_err) => {
            eprintln!("{} Could not run {}. Is it installed?", "Error: ".red().bold(), command.get_program().to_str().unwrap());
            exit(1);
        }
    }.wait()
}

fn build_command(sub_matches: &ArgMatches) {
    let cmake_path = to_str(check_path(get_match(sub_matches, "PATH")));

    let target = get_match(sub_matches, "target");
    let profile = get_match(sub_matches, "profile");
    let eth = get_match(sub_matches, "eth");
    let flash = get_match(sub_matches, "flash");
    
    let t: &str;
    if target == "nucleo" {
        t = "NUCLEO";
    } else {
        t = "BOARD";
    }

    let p: &str;
    if profile == "debug" {
        p = 
        "-g3";
    } else {
        p = 
        "-g0";
    }

    let e: &str;
    if eth == "true" {
        e = "HAL_ETH_MODULE_ENABLED";
    } else {
        e = "OFF";
    }


    let mut cmake_command = process::Command::new("cmake");
    match run_command(cmake_command
        .arg(format!("-DCMAKE_TOOLCHAIN_FILE=arm-none-eabi.cmake"))
        .arg("-S")
        .arg(cmake_path)
        .arg("-B")
        .arg(format!("{cmake_path}/build"))
        .arg(format!("-D{t}=ON"))
        .arg(format!("-DPROFILE={p}"))
        .arg(format!("-D{e}=ON"))) {

        Ok(res) => {
            if res.success() {
                println!("\n\n{}", "Cmake succeeded!".green().bold());
            } else {
                println!("\n\n{}", "Cmake failed!".red().bold());
            }
        },
        Err(err) => {
            err_println("Could not run cmake. Is it installed?");
        }
    }

    let mut make_command = process::Command::new("make");
    match run_command(make_command
        .arg("-j16")
        .arg("-C")
        .arg(format!("{cmake_path}/build"))
        .arg("all")) {

        Ok(res) => {
            if res.success() {
                println!("{}", "Make succeeded!".green().bold());
            } else {
                println!("{}", "Make failed!".red().bold());
            }
        },
        Err(err) => {
            err_println("Could not run make. Is it installed?");
        }
    }

    let current_dir = current_dir().unwrap();
    let project_name = match Path::new(cmake_path).file_name() {
        Some(res) => match res.to_str() {
            Some(res) => res,
            None => {
                eprintln!("{}: {:?} path terminates in .. ", "Error: ".red().bold(), res);
                exit(1)
            }
        },
        None => {
            current_dir.file_name().unwrap().to_str().unwrap()
        }
    };


    let mut objcopy_command = process::Command::new("arm-none-eabi-objcopy");
    match run_command(objcopy_command
        .arg("-O")
        .arg("binary")
        .arg(format!("{cmake_path}/build/{project_name}.elf"))
        .arg(format!("{cmake_path}/build/{project_name}.bin"))) {

        Ok(res) => {
            if res.success() {
                println!("{}", "Objcopy succeeded!".green().bold());
            } else {
                println!("{}", "Objcopy failed!".red().bold());
            }
        },
        Err(err) => {
            err_println("Could not run objcopy. Is it installed?");
        }
    }

    println!("\n\n{}", "Build succeeded!".green().bold());
    println!("\n{:>15}: {:<15}\n{:>15}: {:<15}\n{:>15}: {:<15}\n","Target".yellow(), target.yellow().bold(), "Profile".yellow(), profile.yellow().bold(), "Ethernet".yellow(), eth.yellow().bold());
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
            err_println("Could not run st-flash. Is it installed? (Check apt install st-link tools)");
        }
    }
}

fn main() {
    let now = Instant::now();

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

        _ => {
            eprintln!("{}{}{}.", "Error: ".red().bold(), "The subcommand specified is not a malva command. See ", "'malva --help'".yellow().bold());
        }
    }

    println!("Elapsed time: {}", format!("{:.2?}", now.elapsed()).yellow().bold());
    // Continued program logic goes here...
}
