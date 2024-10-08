use std::{env::current_dir, process};

use cargo_override::{run, Cli};

use clap::Parser;

fn main() {
    let args = Cli::parse();

    if let Err(e) = run(&current_dir().unwrap(), args) {
        eprintln!("error: {e:?}");
        process::exit(101)
    }
}
