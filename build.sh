#!/bin/bash

set -e

# build aarch64-apple-darwin
TARGET=aarch64-apple-darwin

cargo +nightly build -Z build-std=std,panic_abort,core,alloc -Z build-std-features=panic_immediate_abort --target $TARGET --release -p libptrscan
cargo +nightly build -Z build-std=std,panic_abort,core,alloc -Z build-std-features=panic_immediate_abort --target $TARGET --release -p command

RELEASE_DIR=RELEASE/$TARGET

mkdir -p $RELEASE_DIR

cp target/$TARGET/release/libptrscan.dylib $RELEASE_DIR
cp target/$TARGET/release/ptrscan $RELEASE_DIR
cp libptrscan/libptrs.h $RELEASE_DIR
cp libptrscan/ptrscan.py $RELEASE_DIR
cp LICENSE.md $RELEASE_DIR

echo "build $TARGET success."

# build aarch64-apple-ios
TARGET=aarch64-apple-ios

cargo +nightly build -Z build-std=std,panic_abort,core,alloc -Z build-std-features=panic_immediate_abort --target $TARGET --release -p libptrscan
cargo +nightly build -Z build-std=std,panic_abort,core,alloc -Z build-std-features=panic_immediate_abort --target $TARGET --release -p command

RELEASE_DIR=RELEASE/$TARGET

cp target/$TARGET/release/libptrscan.dylib $RELEASE_DIR
cp target/$TARGET/release/ptrscan $RELEASE_DIR
cp libptrscan/libptrs.h $RELEASE_DIR
cp libptrscan/ptrscan.py $RELEASE_DIR
cp LICENSE.md $RELEASE_DIR

echo "build $TARGET success."

# build x86_64-unknown-linux-gnu
TARGET=x86_64-unknown-linux-gnu

cargo +nightly build -Z build-std=std,panic_abort,core,alloc -Z build-std-features=panic_immediate_abort --target $TARGET --release -p libptrscan
cargo +nightly build -Z build-std=std,panic_abort,core,alloc -Z build-std-features=panic_immediate_abort --target $TARGET --release -p command

RELEASE_DIR=RELEASE/$TARGET

cp target/$TARGET/release/libptrscan.so $RELEASE_DIR
cp target/$TARGET/release/ptrscan $RELEASE_DIR
cp libptrscan/libptrs.h $RELEASE_DIR
cp libptrscan/ptrscan.py $RELEASE_DIR
cp LICENSE.md $RELEASE_DIR

echo "build $TARGET success."

# build aarch64-linux-android
TARGET=aarch64-linux-android

cargo +nightly build -Z build-std=std,panic_abort,core,alloc -Z build-std-features=panic_immediate_abort --target $TARGET --release -p libptrscan
cargo +nightly build -Z build-std=std,panic_abort,core,alloc -Z build-std-features=panic_immediate_abort --target $TARGET --release -p command

RELEASE_DIR=RELEASE/$TARGET

cp target/$TARGET/release/libptrscan.so $RELEASE_DIR
cp target/$TARGET/release/ptrscan $RELEASE_DIR
cp libptrscan/libptrs.h $RELEASE_DIR
cp libptrscan/ptrscan.py $RELEASE_DIR
cp LICENSE.md $RELEASE_DIR

echo "build $TARGET success."
