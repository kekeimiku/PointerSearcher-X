#!/bin/bash

set -e

TARGET=$1

cargo +nightly build -Z build-std=std,panic_abort,core,alloc -Z build-std-features=panic_immediate_abort --target $TARGET --release -p libptrscan

