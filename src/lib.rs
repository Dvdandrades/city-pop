use std::fmt;
use std::io;
use std::error::Error;
use std::io::Read;
use serde::Deserialize;
use unicase::UniCase;
use memmap2::Mmap;
use std::fs;
use rayon::prelude::*;
use std::path::Path;
use memchr::memchr;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Row {
    country: String,
    city: String,
    population: Option<u64>,
}

#[derive(Debug)]
pub struct PopulationCount {
    pub city: String,
    pub country: String,
    pub count: u64,
}

#[derive(Debug)]
pub enum CliError {
    Io(io::Error),
    Csv(csv::Error),
    NotFound,
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CliError::Io(ref err) => err.fmt(f),
            CliError::Csv(ref err) => err.fmt(f),
            CliError::NotFound => write!(f, "No matching cities with a population were found."),
        }
    }
}

impl fmt::Display for PopulationCount {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}, {}: {}", self.city, self.country, self.count)
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

pub fn search<R: io::Read>(reader: R, city: &str) -> Result<Vec<PopulationCount>, CliError> {
    let mut rdr = csv::Reader::from_reader(reader);
    let city = UniCase::new(city);

    let found: Vec<PopulationCount> = rdr.deserialize::<Row>()
                                        .filter_map(|result| result.ok())
                                        .filter(|row| UniCase::new(row.city.as_str()) == city)
                                        .filter_map(|row| {
                                            row.population.map(|count| PopulationCount {
                                                city: row.city,
                                                country: row.country,
                                                count,
                                            })
                                        }).collect();
    
    if found.is_empty() {
        Err(CliError::NotFound)
    } else {
        Ok(found)
    }
}

pub fn search_file(path: &Path, city: &str) -> Result<Vec<PopulationCount>, CliError> {
    let file = fs::File::open(path)?;
    let mmap = unsafe { Mmap::map(&file)? };

    let header_end = mmap.iter()
                                .position(|&b| b == b'\n')
                                .map(|p| p + 1)
                                .unwrap_or(mmap.len());

    let header = &mmap[..header_end];
    let data = &mmap[header_end..];

    let n_threads = rayon::current_num_threads();
    let chunk_size = (data.len() / n_threads).max(64 * 1024);
    let chunks = split_at_newlines(data, chunk_size);

    let city = UniCase::new(city);

    let found: Vec<PopulationCount> = chunks.into_par_iter()
                                            .flat_map(|chunk| {
                                                let reader = header.chain(chunk);
                                                let mut rdr = csv::Reader::from_reader(reader);
                                                rdr.deserialize::<Row>()
                                                .filter_map(|r| r.ok())
                                                .filter(|row| UniCase::new(row.city.as_str()) == city)
                                                .filter_map(|row| {
                                                    row.population.map(|count| PopulationCount {
                                                        city: row.city,
                                                        country: row.country,
                                                        count,
                                                    })
                                                }).collect::<Vec<_>>()
                                            }).collect();
        if found.is_empty() {
            Err(CliError::NotFound)
        } else {
            Ok(found)
        }
}

fn split_at_newlines(data: &[u8], chunk_size: usize) -> Vec<&[u8]> {
    let mut chunks = Vec::new();
    let mut start = 0;

    while start < data.len() {
        let end_hint = (start + chunk_size).min(data.len());
        
        let end = if end_hint < data.len() {
            memchr(b'\n', &data[end_hint..])
                .map(|rel| end_hint + rel + 1)
                .unwrap_or(data.len())
        } else {
            data.len()
        };
        chunks.push(&data[start..end]);
        start = end;
    }
    chunks
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn csv(content: &str) -> Cursor<Vec<u8>> {
        Cursor::new(content.as_bytes().to_vec())
    }

    #[test]
    fn found_city() {
        let data = "Country,City,Population\nES,Madrid,3200000\nES,Barcelona,1600000\n";
        let result = search(csv(data), "madrid").unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].city, "Madrid");
        assert_eq!(result[0].count, 3_200_000);
    }

    #[test]
    fn not_found_city() {
        let data = "Country,City,Population\nES,Madrid,3200000\n";
        let result = search(csv(data), "Valencia");
        assert!(matches!(result, Err(CliError::NotFound)));
    }

    #[test]
    fn search_is_case_insensitive() {
        let data = "Country,City,Population\nES,Madrid,3200000\n";
        assert!(search(csv(data), "MADRID").is_ok());
    }

    #[test]
    fn unicode_search() {
        let data = "Country,City,Population\nTR,İstanbul,15000000\n";
        let result = search(csv(data), "İstanbul").unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].count, 15_000_000);
    }

    #[test]
    fn ignore_empty_rows() {
        let data = "Country,City,Population\nES,Madrid,\nES,Madrid,3200000\n";
        let result = search(csv(data), "Madrid").unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].count, 3_200_000);
    }

    #[test]
    fn multiple_matches() {
        let data = "Country,City,Population\nUS,Springfield,50000\nUS,Springfield,60000\n";
        let result = search(csv(data), "Springfield").unwrap();
        assert_eq!(result.len(), 2);
    }
}