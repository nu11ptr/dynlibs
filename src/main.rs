use std::env;
use std::error::Error;
use std::ffi::OsStr;
use std::path::PathBuf;
use std::process;

use dynlibs::DynLibs;

fn print_usage(program: &OsStr) {
    eprintln!("Usage: {} <binary>", program.display());
}

fn run() -> Result<(), Box<dyn Error>> {
    let mut args = env::args_os();

    if args.len() != 2 {
        print_usage(&args.next().ok_or("Launched executable path not found")?);
        process::exit(1);
    }

    let binary_path: PathBuf = args.nth(1).ok_or("No binary path provided")?.into();
    let dyn_libs = DynLibs::from_path(&binary_path)?;
    println!("Binary type: {}\n", dyn_libs.binary_type);
    println!("Dynamic libraries:");
    for lib in dyn_libs.dyn_libs {
        println!("\t{}", lib);
    }
    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}
