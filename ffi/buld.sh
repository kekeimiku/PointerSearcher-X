#!/bin/bash

set -e 

echo '#include <stdint.h>' > ptrsx.h
echo >> ptrsx.h
cbindgen >> ptrsx.h

clang main.c -o main ../target/aarch64-apple-darwin/release/libffi.so