package main

import "C"
import (
	"log"

	"github.com/bluele/hypermint/pkg/contract"
	"github.com/ethereum/go-ethereum/common"
)

var (
	processManager = new(ProcessManager)
	zeroAddress common.Address
)

//export __init_process
func __init_process() int {
	pid, err := processManager.CreateProcess()
	if err != nil {
		log.Println("__init_process:", err)
		return -1
	}
	return pid
}

//export __destroy_process
func __destroy_process() int {
	pid := processManager.CurrentProcessID()
	err := processManager.DestroyProcess(pid)
	if err != nil {
		log.Println(err)
		return -1
	}
	return 0
}

//export __init_contract_address
func __init_contract_address(ptr uintptr, len C.int) int {
	ps, err := processManager.CurrentProcess()
	if err != nil {
		log.Println(err)
		return -1
	}
	if ps.initialized {
		return -1
	}
	var addr common.Address
	copy(addr[:], NewReader(ptr, int(len)).Read())
	ps.InitContractAddress(addr)
	return 0
}

//export __init_sender
func __init_sender(ptr uintptr, len C.int) int {
	ps, err := processManager.CurrentProcess()
	if err != nil {
		log.Println(err)
		return -1
	}
	if ps.initialized {
		return -1
	}
	copy(ps.sender[:], NewReader(ptr, int(len)).Read())
	return 0
}

//export __init_push_arg
func __init_push_arg(ptr uintptr, len C.int) int {
	ps, err := processManager.CurrentProcess()
	if err != nil {
		log.Println(err)
		return -1
	}
	if ps.initialized {
		return -1
	}
	ps.args.PushBytes(NewReader(ptr, int(len)).Read())
	return 0
}

//export __init_done
func __init_done() int {
	ps, err := processManager.CurrentProcess()
	if err != nil {
		log.Println(err)
		return -1
	}
	if ps.initialized {
		return -1
	}
	ps.initialized = true
	return 0
}

//export __clear
func __clear() int {
	ps, err := processManager.CurrentProcess()
	if err != nil {
		log.Println(err)
		return -1
	}
	copy(ps.sender[:], zeroAddress[:])
	ps.args = contract.Args{}
	ps.initialized = false
	return 0
}

//export __commit_state
func __commit_state() int {
	ps, err := processManager.CurrentProcess()
	if err != nil {
		log.Println(err)
		return -1
	}
	if _, err := ps.db.Commit(); err != nil {
		log.Println("__commit_state:", err)
		return -1
	}
	return 0
}

//export __get_return_value
func __get_return_value(offset C.int, ptr uintptr, len C.int) int {
	ps, err := processManager.CurrentProcess()
	if err != nil {
		log.Println(err)
		return -1
	}
	return contract.WriteBuf(ps, NewWriter(ptr, int(len)), int(offset), ps.res)
}

//export __get_event
func __get_event(namePtr uintptr, nameLen C.int, idx C.int, offset C.int, bufPtr uintptr, bufLen C.int) int {
	ps, err := processManager.CurrentProcess()
	if err != nil {
		log.Println(err)
		return -1
	}
	evm := make(map[string][]*contract.Event)
	for _, ev := range ps.events {
		evm[string(ev.Name)] = append(evm[string(ev.Name)], ev)
	}
	name := string(NewReader(namePtr, int(nameLen)).Read())
	evs := evm[name]
	if len(evs) <= int(idx) {
		return -1
	}
	return contract.WriteBuf(ps, NewWriter(bufPtr, int(bufLen)), int(offset), evs[int(idx)].Value)
}

//export __get_sender
func __get_sender(ptr uintptr, len C.int) int {
	ps, err := processManager.CurrentProcess()
	if err != nil {
		log.Println(err)
		return -1
	}
	return contract.GetSender(ps, NewWriter(ptr, int(len)))
}

//export __get_arg
func __get_arg(idx, offset C.int, ptr uintptr, len C.int) int {
	ps, err := processManager.CurrentProcess()
	if err != nil {
		log.Println(err)
		return -1
	}
	return contract.GetArg(ps, int(idx), int(offset), NewWriter(ptr, int(len)))
}

//export __set_response
func __set_response(ptr uintptr, len C.int) int {
	ps, err := processManager.CurrentProcess()
	if err != nil {
		log.Println(err)
		return -1
	}
	return contract.SetResponse(ps, NewReader(ptr, int(len)))
}

//export __log
func __log(ptr uintptr, len C.int) int {
	ps, err := processManager.CurrentProcess()
	if err != nil {
		log.Println(err)
		return -1
	}
	return contract.Log(ps, NewReader(ptr, int(len)))
}

//export __read_state
func __read_state(keyPtr uintptr, keyLen, offset C.int, bufPtr uintptr, bufLen C.int) int {
	ps, err := processManager.CurrentProcess()
	if err != nil {
		log.Println(err)
		return -1
	}
	key := NewReader(keyPtr, int(keyLen))
	buf := NewWriter(bufPtr, int(bufLen))
	return contract.ReadState(ps, key, int(offset), buf)
}

//export __write_state
func __write_state(keyPtr uintptr, keyLen C.int, valPtr uintptr, valLen C.int) int {
	ps, err := processManager.CurrentProcess()
	if err != nil {
		log.Println(err)
		return -1
	}
	key := NewReader(keyPtr, int(keyLen))
	val := NewReader(valPtr, int(valLen))
	return contract.WriteState(ps, key, val)
}

//export __get_mutex
func __get_mutex(pid int) int {
	processManager.GetMutex(pid)
	return 0
}

//export __release_mutex
func __release_mutex() int {
	processManager.ReleaseMutex()
	return 0
}

//export __ecrecover
func __ecrecover(
	h uintptr, hLen C.int,
	v uintptr, vLen C.int,
	r uintptr, rLen C.int,
	s uintptr, sLen C.int,
	buf uintptr, bufLen C.int,
) int {
	ps, err := processManager.CurrentProcess()
	if err != nil {
		log.Println(err)
		return -1
	}
	return contract.ECRecover(
		ps,
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
	ps, err := processManager.CurrentProcess()
	if err != nil {
		log.Println(err)
		return -1
	}
	return contract.ECRecoverAddress(
		ps,
		NewReader(h, int(hLen)),
		NewReader(v, int(vLen)),
		NewReader(r, int(rLen)),
		NewReader(s, int(sLen)),
		NewWriter(buf, int(bufLen)),
	)
}

//export __emit_event
func __emit_event(
	ev uintptr,
	evLen C.int,
	data uintptr,
	dataLen C.int,
) int {
	ps, err := processManager.CurrentProcess()
	if err != nil {
		log.Println(err)
		return -1
	}
	return contract.EmitEvent(
		ps,
		NewReader(ev, int(evLen)),
		NewReader(data, int(dataLen)),
	)
}

//export __push_contract_state
func __push_contract_state(
	addrPtr uintptr,
	addrLen C.int,
) int {
	ps, err := processManager.CurrentProcess()
	if err != nil {
		log.Println(err)
		return -1
	}
	ps.PushState(NewReader(addrPtr, int(addrLen)))
	return 0
}

//export __pop_contract_state
func __pop_contract_state() int {
	ps, err := processManager.CurrentProcess()
	if err != nil {
		log.Println(err)
		return -1
	}
	ps.PopState()
	return 0
}

func main() {}
