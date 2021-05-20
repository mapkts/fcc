# fcc

[![Crates.io](https://img.shields.io/crates/v/fcc?style=flat-square)](https://crates.io/crates/fcc)
[![Linux build status](https://travis-ci.org/mapkts/fcc.svg?branch=master)](https://travis-ci.org/mapkts/fcc)
[![Windows build status](https://ci.appveyor.com/api/projects/status/github/mapkts/fcc?svg=true)](https://ci.appveyor.com/project/mapkts/fcc)

A command line utility for file concatenation, featuring:

- Accepts input files from either `STDIN` or arg `-i`.
- Writes concatenation result to either `STDOUT` or a specific file given by arg `-o`.
- Allows you to skip unwanted contents of each source from either start or end.
- Allows you to fill some paddings before, between and/or after each source.
- Allows you to force the presence of ending newlines after each source.

See `fcc --help` for more help information on how to use this command line utility. And if you want a rust library that provides similar functionalities, see [admerge](https://crates.io/crates/admerge).

## Examples

Assumes we have three text files `1.txt`, `2.txt` and `3.txt` in current working directory.

The content of `1.txt` is:

```bash
111 112 113
121 122 123
131 132 133
```

The content of `2.txt` is:

```bash
211 212 213
221 222 223
231 232 233
```

The content of `3.txt` is:

```bash
311 312 313
321 322 323
331 332 333
```

### Concatenate them without configurations.

```bash
find [1-3].txt | fcc
```

or

```bash
echo [1-3].txt | fcc
```

or

```bash
fcc -i 1.txt 2.txt 3.txt
```

will print the following text to stdout

```bash
111 112 113
121 122 123
131 132 133211 212 213
221 222 223
231 232 233311 312 313
321 322 323
331 332 333
```

### Concatenate them with `--newline`

```bash
echo [1-3].txt | fcc -n
```

will print the following text to stdout:

```bash
111 112 113
121 122 123
131 132 133
211 212 213
221 222 223
231 232 233
311 312 313
321 322 323
331 332 333

```

### Concatenate them with `skip-head=1` and `skip-tail=1`

```bash
echo [1-3].txt | fcc -n --skip-head=1 --skip-tail=1
```

will print the following text to stdout:

```bash
121 122 123
221 222 223
321 322 323

```

### Concatenate them with `newline` and `--headonce`

```bash
echo [1-3].txt | fcc -n --headonce
```

will print the following text to stdout:

```bash
111 112 113
121 122 123
131 132 133
221 222 223
231 232 233
321 322 323
331 332 333

```

### Concatenate them with `--newline` and `padding="padding between\n"`

```bash
echo [1-3].txt | fcc -n --padding="padding between
"
```

will print the following text to stdout:

```bash
111 112 113
121 122 123
131 132 133
padding between
211 212 213
221 222 223
231 232 233
padding between
311 312 313
321 322 323
331 332 333

```

## Installation

Binaries for Windows, Linux and macOS are available [from Github](https://github.com/mapkts/fcc/releases/latest).

You can also compile the binary from source using [Cargo](https://www.rust-lang.org/tools/install):

```bash
git clone git://github.com/mapkts/fcc
cd fcc
cargo build --release
```

Compilation will probably take a few minutes depending on your machine. The
binary will end up in `./target/release/fcc`.

## License

`fcc` is distributed under the terms of either the MIT license or the Apache License (Version 2.0).

See the [LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT) files in this repository for more information.
