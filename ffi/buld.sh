#!/bin/bash

set -e 

cbindgen --output ptrsx.h

clang main.c -o main -L ../target/aarch64-apple-darwin/release -lffi