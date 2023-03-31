[![Crates.io](https://img.shields.io/crates/d/seqrepo.svg)](https://crates.io/crates/seqrepo)
[![Crates.io](https://img.shields.io/crates/v/seqrepo.svg)](https://crates.io/crates/seqrepo)
[![Crates.io](https://img.shields.io/crates/l/seqrepo.svg)](https://crates.io/crates/seqrepo)
[![CI](https://github.com/bihealth/seqrepo-rs/actions/workflows/rust.yml/badge.svg)](https://github.com/bihealth/seqrepo-rs/actions/workflows/rust.yml)
[![codecov](https://codecov.io/gh/bihealth/seqrepo-rs/branch/main/graph/badge.svg?token=aZchhLWdzt)](https://codecov.io/gh/bihealth/seqrepo-rs)

# seqrepo-rs

This is a port of [biocommons/seqrepo](https://github.com/biocommons/seqrepo) to the Rust programming language.

At the moment, only read access has been implemented.
For downloading etc., you will have to use the Python package.

## Running the CLI Example

The library ships with an example called `cli` that you can use to query a seqrepo.

```
# cargo run --example cli -- --help
```
