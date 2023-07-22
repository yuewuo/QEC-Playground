#!/bin/sh
set -ex

cargo clean
cargo clippy -- -Dwarnings  # A collection of lints to catch common mistakes and improve your Rust code.

cargo test --no-run
cargo test --no-run --release
cargo build
cargo build --release
