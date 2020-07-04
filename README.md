# fcc

[![Crates.io](https://img.shields.io/crates/v/fcc?style=flat-square)](https://crates.io/crates/fcc)
[![Linux build status](https://travis-ci.org/mapkts/fcc.svg?branch=master)](https://travis-ci.org/mapkts/fcc)
[![Windows build status](https://ci.appveyor.com/api/projects/status/github/mapkts/fcc?svg=true)](https://ci.appveyor.com/project/mapkts/fcc)

`fcc` is a command line program simply for concatenating files (with some options). Besides, it provides both a library that exposes the same functionality at the command line.

### Example of command line utility

Assumes we have three tabular csv files that contain identical headers. We want to join all the contents of them
and preserve only one header (-H). Meanwhile, we want to make sure all files should end with a newline (-n).

```bash
find [1-3].csv | fcc -nH
```

or

```bash
fcc -nH -i 1.csv 2.csv 3.csv
```

will result the following

```
(header)
(contents of 1.csv)
(contents of 2.csv)
(contents of 3.csv)
```

See `fcc --help` for more help information.

### Documentation

For detailed documentation, see [https://docs.rs/fcc](https://docs.rs/fcc).

### Installation

Binaries for Windows, Linux and macOS are available [from Github](https://github.com/mapkts/fcc/releases/latest).

You can also compile the binary from source using [Cargo](https://www.rust-lang.org/tools/install):

```bash
git clone git://github.com/mapkts/fcc
cd fcc
cargo build --release
```
Compilation will probably take a few minutes depending on your machine. The
binary will end up in `./target/release/fcc`.

### License

`fcc` is distributed under the terms of both the MIT license and the Apache License (Version 2.0).

See the [LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT) files in this repository for more information.