use failure::Error;
use solution::{Evaluation, Solution};
use std::{
    path::PathBuf, process::{Command, Stdio}, time::{Duration, Instant},
};
use tokio::prelude::*;
use tokio_core::reactor::{Handle};
use tokio_io;
use tokio_process::{CommandExt};

pub fn solve_async(
    solver: &PathBuf,
    problem: String,
    handle: Handle,
) -> impl Future<Item = Evaluation, Error = Error> {
    let mut command = Command::new("java");
    command
        .arg("-jar")
        .arg(solver)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped());

    future::lazy(move || {
        let mut child = command
            .spawn_async(&handle)
            .expect("Failed to spawn child process");

        let stdin = child.stdin().take().expect("Failed to open stdin");
        let start = Instant::now();

        tokio_io::io::write_all(stdin, problem)
            .map(move |_| (child, start))
            .and_then(|(child, start)| child.wait_with_output().map(move |c| (c, start)))
            .map(|(output, start)| {
                let duration = Instant::now().duration_since(start);
                (output, duration)
            })
            .deadline(start + Duration::from_secs(300))
    }).from_err()
        .and_then(|(output, duration)| {
            let output = String::from_utf8_lossy(&output.stdout);
            output
                .parse::<Solution>()
                .map(|solution| (solution, duration))
        })
        .map(move |(mut solution, duration)| solution.evaluate(duration))
}
