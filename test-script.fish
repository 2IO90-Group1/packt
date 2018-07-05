#! /usr/bin/fish

cd ~/dev/dbl-algorithms/packt/packt-core
for f in ../testcases/*.txt
    echo $f
    cargo r --release --bin packt-solve -- ../packt-gtk/solver.jar $f ../optimal.csv
end
