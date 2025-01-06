build:
	cargo build
run:
	cargo run $(FILEPATH)
test:
	cargo test
check:
	cargo check
lint:
	cargo clippy -- -D warnings

.PHONY: all run test check lint
