#!/bin/bash

for sq in $(cat data/main_classes/latin_mc6.txt);
do 
    echo $sq > data/cs/$sq-counts.txt
    echo $sq | cat - data/cs/$sq.cs | cargo r -r decode-cs | cargo r -r count-entries >> data/cs/$sq-counts.txt
done 