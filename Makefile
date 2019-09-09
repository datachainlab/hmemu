build:
	go build -o ./build/libhm.so -buildmode=c-shared ./runtime

lint-tools:
	rustup component add clippy
	rustup component add rustfmt

lint:
	cargo fmt -- --check
	cargo clippy -- -D warnings

.PHONY: build lint
