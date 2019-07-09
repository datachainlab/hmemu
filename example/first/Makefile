CONTRACT_NAME?=first
CONTRACT_CODE?=./$(CONTRACT_NAME).min.wasm

build:
	cargo build --target=wasm32-unknown-unknown
	wasm-gc ./target/wasm32-unknown-unknown/debug/$(CONTRACT_NAME).wasm -o $(CONTRACT_CODE)

test:
	cargo test

clean:
	rm -rf ./target $(CONTRACT_CODE)
