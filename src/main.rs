use std::fmt;
use std::io;
use std::path::Path;
use std::fs;
use std::process;
use serde::Deserialize;
use std::error::Error;
use clap::Parser;


#[derive(Debug, Parser)]
#[command(name="city-pop", version, about="Search for city populations in CSV files")]
struct Args {
    /// Name of the city to search
    city: String,
    /// Path to the data file (if omitted, standard input is used)
    data_path: Option<String>,
    /// Do not print errors if the city is not found
    #[arg(short, long)]
    quiet: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all="PascalCase")]
struct Row {
    country: String,
    city: String,
    population: Option<u64>, 
}

struct PopulationCount {
    city: String,
    country: String,
    count: u64,
}

#[derive(Debug)]
enum CliError {
    Io(io::Error),
    Csv(csv::Error),
    NotFound,
}

macro_rules! fatal {
    ($($tt:tt)*) => {{
        use std::io::Write;
        let _ = writeln!(&mut ::std::io::stderr(), $($tt)*);
        ::std::process::exit(1)
    }}
}

impl fmt::Display for CliError {
    fn fmt(&self, f:&mut fmt::Formatter) -> fmt::Result {
        match *self {
            CliError::Io(ref err) => err.fmt(f),
            CliError::Csv(ref err) => err.fmt(f),
            CliError::NotFound => write!(f, "No matching cities with a population were found."),
        }
    }
}

impl Error for CliError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match *self {
            CliError::Io(ref err) => Some(err),
            CliError::Csv(ref err) => Some(err),
            CliError::NotFound => None,
        }
    }
}

impl From<io::Error> for CliError {
    fn from(err: io::Error) -> CliError {
        CliError::Io(err)
    }
}

impl From<csv::Error> for CliError {
    fn from(err: csv::Error) -> CliError {
        CliError::Csv(err)
    }
}

fn search<P: AsRef<Path>>(file_path: &Option<P>, city: &str) 
        -> Result<Vec<PopulationCount>, CliError> {
    let input: Box<dyn io::Read> = match file_path {
        None => Box::new(io::stdin()),
        Some(file_path) => Box::new(fs::File::open(file_path)?),
    };
    let mut rdr = csv::Reader::from_reader(input);
    let found : Vec<PopulationCount> = rdr.deserialize::<Row>()
            .filter_map(|result| result.ok())
            .filter(|row| row.city == city)
            .filter_map(|row| {
                row.population.map(|count| PopulationCount {
                    city: row.city,
                    country: row.country,
                    count,
                })
            })
            .collect();
    if found.is_empty() {
        Err(CliError::NotFound)
    } else {
        Ok(found)
    }
} 

fn main() {
    let args: Args = Args::parse();

    match search(&args.data_path, &args.city) {
        Err(CliError::NotFound) if args.quiet => process::exit(1),
        Err(err) => fatal!("{}", err),
        Ok(pops) => for pop in pops {
            println!("{}, {}: {:?}", pop.city, pop.country, pop.count);
        }
    }
}