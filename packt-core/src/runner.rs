use failure::Error;
use problem::Problem;
use solution::{Evaluation, Solution};
use std::{
    path::PathBuf,
    process::{Command, Stdio},
    time::{Duration, Instant},
};
use tokio::prelude::*;
use tokio_core::reactor::Handle;
use tokio_io;
use tokio_process::CommandExt;

pub fn solve_async(
    solver: &PathBuf,
    problem: Problem,
    handle: Handle,
    delta: Duration,
) -> impl Future<Item = Evaluation, Error = Error> {
    let mut command = Command::new("java");
    command
        .arg("-jar")
        .arg(solver)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped());

    let input = problem.to_string();
    future::lazy(move || {
        let mut child = command
            .spawn_async(&handle)
            .expect("Failed to spawn child process");

        let stdin = child.stdin().take().expect("Failed to open stdin");
        let start = Instant::now();

        tokio_io::io::write_all(stdin, input)
            .map(move |_| (child, start))
            .and_then(|(child, start)| child.wait_with_output().map(move |c| (c, start)))
            .map(|(output, start)| {
                let duration = Instant::now().duration_since(start);
                (output, duration)
            })
            .deadline(start + delta)
    }).from_err()
        .and_then(|(output, duration)| {
            let output = String::from_utf8_lossy(&output.stdout);
            output.parse::<Solution>().map(|mut solution| {
                solution.source(problem);
                (solution, duration)
            })
        })
        .and_then(move |(mut solution, duration)| solution.evaluate(duration))
}
