build:
	go build -o ./build/libhm.so -buildmode=c-shared ./lib

.PHONY: build
