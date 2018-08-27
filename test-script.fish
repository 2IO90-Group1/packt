#! /usr/bin/fish

cd ~/dev/dbl-algorithms/packt/packt-core
cargo build --bin packt-solve --release
cd ../
./target/release/packt-solve packt-gtk/solver.jar testcases skyline-minimal.csv
