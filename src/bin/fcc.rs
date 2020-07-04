extern crate fcc;
use clap::{App, Arg};
use fcc::Concat;

use std::fs::OpenOptions;
use std::io::{self, Read};

macro_rules! werr {
    ($($arg:tt)*) => {
        use std::io::Write;
        (writeln!(&mut std::io::stderr(), $($arg)*)).unwrap();
    }
}

fn main() {
    if let Err(e) = run() {
        werr!("fcc: {}", e);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let crate_name = env!("CARGO_PKG_NAME");
    let crate_description = env!("CARGO_PKG_DESCRIPTION");
    let crate_author = env!("CARGO_PKG_AUTHORS");
    let crate_version = env!("CARGO_PKG_VERSION");

    let matches = App::new(crate_name)
        .about(crate_description)
        .author(crate_author)
        .version(crate_version)
        .arg(
            Arg::with_name("input")
                .short("i")
                .long("input")
                .takes_value(true)
                .multiple(true)
                .help("Provides some files to this command"),
        )
        .arg(
            Arg::with_name("output")
                .short("o")
                .long("output")
                .takes_value(true)
                .help("Writes output to a specific <file> instead of STDOUT"),
        )
        .arg(Arg::with_name("newline").short("n").long("newline").help(
            "Appends a newline (\\n) after each file if file is not already ended with newline",
        ))
        .arg(
            Arg::with_name("header")
                .short("H")
                .long("header")
                .help("Preserves the header of the first passed-in file and drops the rest"),
        )
        .arg(
            Arg::with_name("skip")
                .short("s")
                .long("skip")
                .takes_value(true)
                .value_name("NUMBER")
                .help("Drops first n lines in each file while concatenating"),
        )
        .arg(
            Arg::with_name("padding")
                .short("p")
                .long("padding")
                .takes_value(true)
                .value_name("STRING")
                .help("Fills some paddings between each file while concatenating"),
        )
        .get_matches();

    // Reads input from cli argument (primary) or `stdin` (fallback).
    let input: Vec<String> = if matches.is_present("input") {
        matches
            .values_of("input")
            .unwrap()
            .map(|v| v.trim().to_owned())
            .collect::<Vec<String>>()
    } else {
        let mut buf = String::new();
        io::stdin().lock().read_to_string(&mut buf)?;

        let paths = buf
            .split(' ')
            .filter(|v| v != &"")
            .map(|v| v.trim().to_owned())
            .collect::<Vec<String>>();

        if paths.len() >= 2 {
            paths
        } else {
            buf.split('\n')
                .filter(|v| v != &"")
                .map(|v| v.trim().to_owned())
                .collect::<Vec<String>>()
        }
    };

    // Reads cli options and builds a `Concat` instance from them.
    let mut concat = Concat::new();
    if matches.is_present("newline") {
        concat.newline(true);
    }
    if matches.is_present("header") {
        concat.header(true);
    }
    if matches.is_present("skip") {
        let n = matches.value_of("skip").unwrap().parse::<usize>()?;
        concat.skip_line(n);
    }
    if matches.is_present("padding") {
        let padding = matches.value_of("padding").unwrap().as_bytes();
        concat.pad_with(padding);
    }
    let concat = concat.open(input);

    // Writes the concatenation result.
    if matches.is_present("output") {
        let path = matches.value_of("output").unwrap();
        let mut file = OpenOptions::new().create(true).write(true).open(path)?;
        concat.write(&mut file)?;
    } else {
        concat.write(&mut io::stdout().lock())?;
    }

    Ok(())
}
