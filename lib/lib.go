package main

import "C"
import (
	"log"

	"github.com/bluele/hypermint/pkg/contract"
)

var (
	globalProcess *Process
	isInitDone    bool
)

//export __init_process
func __init_process() int {
	if isInitDone {
		return -1
	}
	var err error
	globalProcess, err = NewProcess()
	if err != nil {
		log.Println("__init_process:", err)
		return -1
	}
	return 0
}

//export __init_sender
func __init_sender(ptr uintptr, len C.int) int {
	if isInitDone {
		return -1
	}
	copy(globalProcess.sender[:], NewReader(ptr, int(len)).Read())
	return 0
}

//export __init_push_arg
func __init_push_arg(ptr uintptr, len C.int) int {
	if isInitDone {
		return -1
	}
	globalProcess.args.PushBytes(NewReader(ptr, int(len)).Read())
	return 0
}

//export __init_done
func __init_done() int {
	if isInitDone {
		return -1
	}
	isInitDone = true
	return 0
}

//export __commit_state
func __commit_state() int {
	_, err := globalProcess.db.Commit()
	if err != nil {
		log.Println("__commit_state:", err)
		return -1
	}
	return 0
}

//export __get_sender
func __get_sender(ptr uintptr, len C.int) int {
	return contract.GetSender(globalProcess, NewWriter(ptr, int(len)))
}

//export __get_arg
func __get_arg(idx C.int, ptr uintptr, len C.int) int {
	return contract.GetArg(globalProcess, int(idx), NewWriter(ptr, int(len)))
}

//export __log
func __log(ptr uintptr, len C.int) int {
	return contract.Log(globalProcess, NewReader(ptr, int(len)))
}

//export __read_state
func __read_state(keyPtr uintptr, keyLen C.int, bufPtr uintptr, bufLen C.int) int {
	key := NewReader(keyPtr, int(keyLen))
	buf := NewWriter(bufPtr, int(bufLen))
	return contract.ReadState(globalProcess, key, buf)
}

//export __write_state
func __write_state(keyPtr uintptr, keyLen C.int, valPtr uintptr, valLen C.int) int {
	key := NewReader(keyPtr, int(keyLen))
	val := NewReader(valPtr, int(valLen))
	return contract.WriteState(globalProcess, key, val)
}

//export __ecrecover
func __ecrecover(
	h uintptr, hLen C.int,
	v uintptr, vLen C.int,
	r uintptr, rLen C.int,
	s uintptr, sLen C.int,
	buf uintptr, bufLen C.int,
) int {
	return contract.ECRecover(
		globalProcess,
		NewReader(h, int(hLen)),
		NewReader(v, int(vLen)),
		NewReader(r, int(rLen)),
		NewReader(s, int(sLen)),
		NewWriter(buf, int(bufLen)),
	)
}

//export __ecrecover_address
func __ecrecover_address(
	h uintptr, hLen C.int,
	v uintptr, vLen C.int,
	r uintptr, rLen C.int,
	s uintptr, sLen C.int,
	buf uintptr, bufLen C.int,
) int {
	return contract.ECRecoverAddress(
		globalProcess,
		NewReader(h, int(hLen)),
		NewReader(v, int(vLen)),
		NewReader(r, int(rLen)),
		NewReader(s, int(sLen)),
		NewWriter(buf, int(bufLen)),
	)
}

func main() {}
