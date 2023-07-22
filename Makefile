all: test check build python

fmt:
	cargo fmt --check

clippy:
	cargo clippy -- -Dwarnings  # A collection of lints to catch common mistakes and improve your Rust code.

clean:
	cargo clean

clean-env: clean fmt clippy

test: clean-env
	cargo clean
	cargo test
	cargo run --release -- test all

build: clean-env
	cargo clean
	cargo test --no-run
	cargo test --no-run --release
	cargo build
	cargo build --release

check: clean-env
	cargo check
	cargo check --release

python: clean-env
	maturin develop
	# pytest tests/python
