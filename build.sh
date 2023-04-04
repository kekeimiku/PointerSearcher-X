#!/bin/bash

set -e

cargo +nightly clippy --fix --allow-dirty --allow-staged

cargo +nightly fmt

cargo build --target aarch64-apple-darwin --release

# cargo +nightly build -Z build-std=std,panic_abort -Z build-std-features=panic_immediate_abort --target aarch64-apple-darwin --release

cp target/aarch64-apple-darwin/release/ups-cli build

codesign -s kk.ups build/ups-cli

ldid -Sentitlements.plist build/ups-cli