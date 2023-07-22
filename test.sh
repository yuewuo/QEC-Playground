#!/bin/sh
set -ex

cargo clean

cargo test

cargo run --release -- test all
