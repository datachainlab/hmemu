build:
	go build -o ./build/libhm.so -buildmode=c-shared ./runtime

.PHONY: build
