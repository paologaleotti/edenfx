build:
	cargo build

debug:
	RUST_LOG=debug cargo run


release:
	cargo build --release

.PHONY: build debug release