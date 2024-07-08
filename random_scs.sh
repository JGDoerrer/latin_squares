#!/bin/bash

cargo r -r random-latin-squares $(shuf -i 1-10000000 -n1)   \
    | head -1                                               \
    | cargo r -r find-scs                                   \
    | tail -n+2                                             \
    | shuf -n1                                              \
    | cargo r -r pretty-print