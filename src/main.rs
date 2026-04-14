use std::fs;
use std::io;
use std::path::PathBuf;
use std::process;
use clap::{Parser, ValueHint};
use city_pop::{search, CliError};

#[derive(Debug, Parser)]
#[command(name="city-pop", version, about="Search for city populations in CSV files")]
struct Args {
    /// Name of the city to search
    city: String,

    /// Path to the data file (if omitted, standard input is used)
    #[arg(value_hint=ValueHint::FilePath)]
    data_path: Option<PathBuf>,

    /// Do not print errors if the city is not found
    #[arg(short, long)]
    quiet: bool,
}

fn main() {
    let args = Args::parse();

    let result = match args.data_path {
        Some(ref path) => match fs::File::open(path) {
            Ok(file) => search(file, &args.city),
            Err(err) => Err(CliError::Io(err)),
        },
        None => search(io::stdin(), &args.city),
    };

    match result {
        Err(CliError::NotFound) if args.quiet => process::exit(1),
        Err(err) => {
            eprintln!("{}", err);
            process::exit(1);
        }
        Ok(pops) => {
            for pop in pops {
                println!("{}", pop);
            }
        }
    }
}