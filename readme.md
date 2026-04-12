# city-pop

`city-pop` is a small Rust command-line program that looks up city populations from a CSV file.
It is a modern adaptation of an example originally published by Andrew Gallant ([@BurntSushi](https://github.com/BurntSushi)), updated to use current Rust idioms, `clap` for argument parsing, and `serde`-based deserialization.

## What It Does

Given a city name, the program searches a CSV dataset and prints every matching city with a recorded population:

```text
andorra la vella, ad: 20430
```

If no matching populated city is found, it returns an error message. With `--quiet`, it exits with status code `1` instead of printing that message.

## Installation

Build it with Cargo:

```bash
cargo build --release
```

Run the compiled binary from `target/release/city-pop`, or use `cargo run` during development.

## Usage

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

Search using the bundled dataset:

```bash
cargo run -- "andorra la vella" data.csv
```

Read from standard input instead of a file:

```bash
cat data.csv | cargo run -- "andorra la vella"
```

Suppress the error message when no result is found:

```bash
cargo run -- --quiet "not-a-city" data.csv
```

## CSV Format

The program deserializes rows using the following CSV headers:

- `Country`
- `City`
- `Population`

Other columns may exist in the file and are ignored. Rows without a population are skipped.

## Notes

- Matching is currently exact and case-sensitive.
- If multiple rows have the same city name and population data, all of them are printed.
- When `[DATA_PATH]` is omitted, the program reads CSV data from standard input.

## Credit

This project is intentionally rooted in Andrew Gallant's teaching style and original CSV/CLI examples. The core idea comes from his work, while this version updates the implementation for a more modern Rust toolchain and dependency stack.
