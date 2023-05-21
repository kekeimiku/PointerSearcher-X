#!/bin/bash

set -e 

echo '#include <stdint.h>' > ptrsx.h
echo >> ptrsx.h
cbindgen >> ptrsx.h

clang main.c -o main -L ../target/aarch64-apple-darwin/release -lffi