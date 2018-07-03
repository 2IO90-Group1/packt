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

use packt_core::{problem::Problem, runner};
use quicli::prelude::*;
use std::{
    env, fs::File, io::{self, BufReader}, path::PathBuf, time::Duration,
};
use tokio::prelude::*;
use tokio_core::reactor::Core;

#[derive(Debug, StructOpt)]
struct Cli {
    /// Solver jar-file to solve with
    #[structopt(parse(from_os_str))]
    solver: PathBuf,

    /// Output file, stdout if not present
    #[structopt(parse(from_os_str))]
    input: Option<PathBuf>,

    #[structopt(flatten)]
    verbosity: Verbosity,
}

main!(|args: Cli, log_level: verbosity| {
    let mut input: Box<dyn io::Read> = match args.input {
        Some(path) => {
            let file = File::open(path)?;
            Box::new(BufReader::new(file))
        }
        None => Box::new(io::stdin()),
    };

    let mut buffer = String::new();
    input.read_to_string(&mut buffer)?;
    let _ = buffer.parse::<Problem>()?;

    let deadline = Duration::from_secs(300);
    let mut core = Core::new().unwrap();

    let vals = [5, 10, 25, 50, 100];
    for (retry, candidates) in iproduct!(&vals, &vals) {
        eprintln!("RETRY = {}, N_HEIGHTS = {}", retry, candidates);
        env::set_var("RETRY", retry.to_string());
        env::set_var("N_HEIGHTS", candidates.to_string());

        let handle = core.handle();
        let child = runner::solve_async(&args.solver, buffer.clone(), handle, deadline);
        let evaluation = core.run(child);
        println!("RETRY = {}, N_HEIGHTS = {}", retry, candidates);
        match evaluation {
            Ok(eval) => {
                println!("{}\n", eval);
            }
            Err(e) => {
                error!("{:?}", e);
                println!("{:?}", e);
            }
        }
    }
});
