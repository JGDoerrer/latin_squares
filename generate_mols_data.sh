#!/bin/bash

for i in $(seq 1 9);
do
    for j in $(seq 2 $(($i - 1)));
    do
        cat "data/main_classes/latin_mc$i.txt" \
            | cargo r -r find-mols $i $j \
            > data/mols/latin_mols_$(echo $i)_$j.txt
    done 
done 