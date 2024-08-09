#!/bin/bash

cargo r -r random $1 $(shuf -i 1-10000000 -n1) \
    | head -1                                 \
    | cargo r -r find-scs -r                  \
    | tail -2 | head -1                       \
    | cargo r -r pretty-print