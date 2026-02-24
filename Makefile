all: lint test doc

test:
	cargo test

lint: fmt clippy

fmt:
	cargo fmt --all

clippy:
	cargo clippy --all-targets -- -D warnings

doc:
	cargo doc --no-deps

.PHONY: all test lint fmt clippy doc
