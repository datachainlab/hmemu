build:
	go build -o ./build/libhm.so -buildmode=c-shared ./runtime

lint:
	cargo fmt -- --check
	cargo clippy -- -D warnings

.PHONY: build lint
