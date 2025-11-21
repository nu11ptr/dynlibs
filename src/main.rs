use std::env;
use std::error::Error;
use std::ffi::OsStr;
use std::path::PathBuf;
use std::process;

use dynlibs::{DynLibEntries, Executable};
use regex::Regex;

fn print_usage(program: &OsStr) {
    eprintln!(
        "Usage: {} [--only|-o <library_regex> ]... <binary>",
        program.display()
    );
}

fn run() -> Result<(), Box<dyn Error>> {
    let mut args = env::args_os();
    let program = args.next().ok_or("Program name not found")?;

    let last_arg = args.next_back().ok_or_else(|| {
        print_usage(&program);
        process::exit(1);
    })?;

    if last_arg == "-h" || last_arg == "-help" || last_arg == "--help" || last_arg == r#"/?"# {
        print_usage(&program);
        process::exit(1);
    } else {
        let mut regexes = Vec::with_capacity(args.len() / 2);

        while let Some(flag) = args.next() {
            if flag == "-o" || flag == "--only" {
                let regex = args.next().ok_or("No regex provided")?;
                regexes.push(Regex::new(&regex.to_string_lossy())?);
            } else {
                return Err(format!("Invalid flag: '{}'", flag.display()).into());
            }
        }

        let binary_path: PathBuf = last_arg.into();
        let dyn_libs = Executable::from_path(&binary_path)?;

        // Display mode
        if regexes.is_empty() {
            println!("Binary type: {}\n", dyn_libs.binary_type);
            println!("Dynamic libraries:");

            match dyn_libs.dyn_libs {
                DynLibEntries::SingleArch(libs) => {
                    for lib in libs.iter() {
                        println!("    {}", lib);
                    }
                }
                DynLibEntries::MultiArch(entries) => {
                    for entry in entries {
                        println!("    Index {}:", entry.index);
                        for lib in entry.dyn_libs.iter() {
                            println!("        {}", lib);
                        }
                    }
                }
            }
        } else {
            // Match mode
            match dyn_libs.dyn_libs {
                DynLibEntries::SingleArch(libs) => {
                    let matches = libs.build_matches(&regexes);
                    if !matches.all_matched() {
                        eprintln!("Some dynamic libraries did not match the provided regexes:");
                        for lib in matches.unmatched {
                            eprintln!("    {}", lib);
                        }
                        process::exit(1);
                    }
                }
                DynLibEntries::MultiArch(entries) => {
                    let matches = entries
                        .iter()
                        .map(|entry| (entry.index, entry.dyn_libs.build_matches(&regexes)));
                    let all_matched = matches.clone().all(|matches| matches.1.all_matched());

                    if !all_matched {
                        eprintln!("Some dynamic libraries did not match the provided regexes:");
                        for (index, matches) in matches {
                            eprintln!("    Index {}:", index);
                            for lib in matches.unmatched {
                                eprintln!("        {}", lib);
                            }
                        }
                        process::exit(1);
                    }
                }
            }
        }
    }

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}
