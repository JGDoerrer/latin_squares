#!/bin/bash

for sq in $(cat data/main_classes/latin_mc7.txt);
do 
    echo $sq | cargo r -r find-all-cs > data/cs/$sq.cs &
done 