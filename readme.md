# city-pop

`city-pop` is a Rust CLI and library for looking up city populations from CSV data.
It supports reading from standard input or, when given a file path, using a memory-mapped and parallelized search path for larger datasets.

The project is a modernized take on an example originally published by Andrew Gallant ([@BurntSushi](https://github.com/BurntSushi)), updated with `clap`, `serde`, `memmap2`, and `rayon`.

## Features

- Exact, case-insensitive city matching
- CSV deserialization with `serde`
- Reads from `stdin` or a file path
- Faster file-backed searches via memory mapping and parallel chunk processing
- Reusable library API in addition to the CLI

## Installation

Build the project with Cargo:

```bash
cargo build --release
```

Run the binary from `target/release/city-pop`, or use `cargo run` while developing.

## CLI Usage

```text
Search for city populations in CSV files

Usage: city-pop [OPTIONS] <CITY> [DATA_PATH]

Arguments:
  <CITY>       Name of the city to search
  [DATA_PATH]  Path to the data file (if omitted, standard input is used)

Options:
  -q, --quiet    Do not print errors if the city is not found
  -h, --help     Print help
  -V, --version  Print version
```

## Examples

Search a CSV file:

```bash
cargo run -- "Andorra la Vella" data.csv
```

Read from standard input:

```bash
cat data.csv | cargo run -- "Andorra la Vella"
```

Return a non-zero exit code without printing the not-found message:

```bash
cargo run -- --quiet "not-a-city" data.csv
```

Typical output:

```text
andorra la vella, ad: 20430
```

If multiple rows match the same city name and have a population, every match is printed.

## CSV Format

The library deserializes rows using these headers:

- `Country`
- `City`
- `Population`

Additional columns are ignored. Rows with an empty `Population` value are skipped.

## Library Usage

Use `search` when you already have a reader, and `search_file` when you want the optimized file-backed path:

```rust
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use city_pop::{search, search_file};

let reader = BufReader::new(File::open("data.csv")?);
let from_reader = search(reader, "madrid")?;

let from_file = search_file(Path::new("data.csv"), "madrid")?;
```

Both functions return a `Vec<PopulationCount>` and report `CliError::NotFound` when no populated matches exist.

## Notes

- Matching is exact but case-insensitive.
- Unicode city names are supported.
- When a file path is provided, the file is opened read-only and searched in newline-aligned chunks in parallel.
- When no data path is provided, the CLI reads CSV content from standard input.

## Credit

This project is intentionally rooted in Andrew Gallant's teaching style and original CSV/CLI examples, with the implementation refreshed for a modern Rust toolchain.
