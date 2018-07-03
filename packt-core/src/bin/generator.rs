extern crate failure;
extern crate log;
extern crate packt_core;
#[macro_use]
extern crate quicli;

use packt_core::problem;
use quicli::prelude::*;
use std::{fs::OpenOptions, io, path::PathBuf};

#[derive(Debug, StructOpt)]
struct Cli {
    /// Amount of rectangles to generate
    #[structopt(long = "count", short = "n")]
    count: usize,

    /// Whether solutions are allowed to rotate rectangles.
    /// Will be generated randomly by default.
    #[structopt(long = "rotation", short = "r")]
    rotation: Option<bool>,

    /// The height to which the solutions are bound.
    /// This value should be greater than or equal to <count>.
    /// Will be generated randomly by default.
    #[structopt(long = "variant", short = "f")]
    variant: Option<problem::Variant>,

    /// Output file, stdout if not present
    #[structopt(help = "Output file, stdout if not present", parse(from_os_str))]
    output: Option<PathBuf>,

    #[structopt(flatten)]
    verbosity: Verbosity,
}

main!(|args: Cli, log_level: verbosity| {
    let n = args.count;
    let variant = args.variant;
    let rotation = args.rotation;
    let problem = problem::generate(n, variant, rotation);

    let mut dest: Box<dyn io::Write> = match args.output {
        Some(path) => Box::new(OpenOptions::new().write(true).create(true).open(path)?),
        None => Box::new(io::stdout()),
    };

    dest.write_all(problem.to_string().as_bytes())?;
});
