#! /usr/bin/fish

cd ~/dev/dbl-algorithms/packt/packt-core
cargo build --bin packt-solve --release
cd ../
for f in testcases/optimal/*.txt
    echo $f
    ./target/release/packt-solve /home/frank/dev/dbl-algorithms/solver/out/artifacts/solver_jar/solver.jar $f optimal.csv
end
./target/release/packt-solve packt-gtk/solver.jar testcases/optimal optimal.csv
