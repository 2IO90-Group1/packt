extern crate failure;
extern crate log;
extern crate packt_core;
#[macro_use]
extern crate quicli;
extern crate tokio;
extern crate tokio_core;
extern crate tokio_io;
extern crate tokio_process;
#[macro_use]
extern crate itertools;
extern crate csv;
extern crate serde;
#[macro_use]
extern crate serde_derive;

use packt_core::{problem::Problem, runner, solution::Evaluation};
use quicli::prelude::*;
use std::{
    env, fs::{File, OpenOptions}, io::{self, BufReader}, path::PathBuf, time::Duration,
};
use tokio::prelude::*;
use tokio_core::reactor::Core;

#[derive(Debug, StructOpt)]
struct Cli {
    /// Solver jar-file to solve with
    #[structopt(parse(from_os_str))]
    solver: PathBuf,

    /// Input file, stdin if not present
    #[structopt(parse(from_os_str))]
    input: Option<PathBuf>,

    /// Output file, stdout if not present
    #[structopt(parse(from_os_str))]
    output: Option<PathBuf>,

    #[structopt(flatten)]
    verbosity: Verbosity,
}


main!(|args: Cli, log_level: verbosity| {
    let filename = args
        .input
        .as_ref()
        .and_then(|pb| pb.file_name().and_then(|f| f.to_str()))
        .unwrap_or_default();

    let mut input: Box<dyn io::Read> = match args.input {
        Some(ref path) => {
            let file = File::open(path)?;
            Box::new(BufReader::new(file))
        }
        None => Box::new(io::stdin()),
    };

    let output: Box<dyn io::Write> = match args.output {
        Some(path) => Box::new(OpenOptions::new().append(true).create(true).open(path)?),
        None => Box::new(io::stdout()),
    };

    let mut writer = csv::Writer::from_writer(output);

    let mut buffer = String::new();
    input.read_to_string(&mut buffer)?;
    let problem = buffer.parse::<Problem>()?;

    let deadline = Duration::from_secs(300);
    let mut core = Core::new().unwrap();

    let handle = core.handle();
    let child = runner::solve_async(&args.solver, problem.clone(), handle, deadline);
    let evaluation = core.run(child);
    let record = Record::new(&problem, evaluation, filename);

    writer.serialize(record)?;
    writer.flush()?;
});

#[derive(Debug, Serialize)]
struct Record<'a> {
    filename: &'a str,
    n: usize,
    variant: String,
    rotation_allowed: bool,
    perfect_packing: bool,
    error: Option<String>,
    container: Option<String>,
    min_area: Option<u64>,
    empty_area: Option<i64>,
    filling_rate: Option<f32>,
    duration: Option<String>,
}

impl<'a> Record<'a> {
    fn new<'b>(
        problem: &'b Problem,
        evaluation: Result<Evaluation>,
        filename: &'a str,
    ) -> Self {
        let &Problem {
            variant,
            allow_rotation,
            ref rectangles,
            ..
        } = problem;
        let n = rectangles.len();

        let (container, min_area, empty_area, filling_rate, duration, error) = match evaluation {
            Ok(eval) => {
                let Evaluation {
                    min_area,
                    empty_area,
                    filling_rate,
                    duration,
                    container,
                    ..
                } = eval;
                (
                    Some(container.to_string()),
                    Some(min_area),
                    Some(empty_area),
                    Some(filling_rate),
                    Some(format!(
                        "{}.{:.3}",
                        duration.as_secs(),
                        duration.subsec_millis(),
                    )),
                    None,
                )
            }
            Err(e) => (None, None, None, None, None, Some(e.to_string())),
        };

        Record {
            filename,
            retry,
            n_candidates,
            n,
            variant: variant.to_string(),
            rotation_allowed: allow_rotation,
            perfect_packing: filename.contains("packt"),
            container,
            min_area,
            empty_area,
            filling_rate,
            duration,
            error,
        }
    }
}
