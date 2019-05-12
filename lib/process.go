package main

import "C"
import (
	"errors"
	"fmt"
	"sync"
	"unsafe"

	"github.com/bluele/hypermint/pkg/abci/store"
	sdk "github.com/bluele/hypermint/pkg/abci/types"
	"github.com/bluele/hypermint/pkg/contract"
	"github.com/bluele/hypermint/pkg/db"
	"github.com/bluele/hypermint/pkg/logger"
	"github.com/ethereum/go-ethereum/common"
	dbm "github.com/tendermint/tendermint/libs/db"
)

var (
	defaultLogger                  = logger.GetDefaultLogger("*:debug").With("module", "process")
	_             contract.Process = new(Process)
)

type ProcessManager struct {
	mu   sync.RWMutex
	cpid int
	pss  []*Process
}

func (pm *ProcessManager) GetMutex() {
	pm.mu.Lock()
}

func (pm *ProcessManager) ReleaseMutex() {
	pm.mu.Unlock()
}

func (pm *ProcessManager) CreateProcess() (int, error) {
	pid := len(pm.pss)
	ps, err := NewProcess()
	if err != nil {
		return -1, err
	}
	pm.pss = append(pm.pss, ps)
	pm.cpid = pid
	return pid, nil
}

func (pm *ProcessManager) DestroyProcess(pid int) error {
	if len(pm.pss) <= pid {
		return fmt.Errorf("not found pid %v", pid)
	}
	pm.pss[pid] = nil
	return nil
}

func (pm *ProcessManager) CurrentProcess() (*Process, error) {
	if len(pm.pss) <= pm.cpid {
		return nil, fmt.Errorf("not found pid %v", pm.cpid)
	}
	ps := pm.pss[pm.cpid]
	if ps == nil {
		return nil, errors.New("deleted process")
	}
	return ps, nil
}

func (pm *ProcessManager) CurrentProcessID() int {
	return pm.cpid
}

func (pm *ProcessManager) SwitchProcess(pid int) {
	pm.cpid = pid
}

type Process struct {
	initialized bool
	db          *db.VersionedDB

	sender common.Address
	args   contract.Args
	res    []byte
}

func NewProcess() (*Process, error) {
	mdb := dbm.NewMemDB()
	cms := store.NewCommitMultiStore(mdb)
	var key = sdk.NewKVStoreKey("main")
	cms.MountStoreWithDB(key, sdk.StoreTypeIAVL, nil)
	if err := cms.LoadLatestVersion(); err != nil {
		return nil, err
	}
	kvs := cms.GetKVStore(key)
	return &Process{db: db.NewVersionedDB(kvs, db.Version{1, 1})}, nil
}

func (p *Process) Sender() common.Address {
	return p.sender
}

func (p *Process) Args() contract.Args {
	return p.args
}

func (p *Process) Call(addr common.Address, entry []byte, args contract.Args) (int, error) {
	panic("not implemented error")
}

func (p *Process) Read(id int) ([]byte, error) {
	panic("not implemented error")
}

func (p *Process) ValueTable() contract.ValueTable {
	panic("not implemented error")
}

func (p *Process) SetResponse(b []byte) {
	p.res = b
}

func (p *Process) State() db.StateDB {
	return p.db
}

func (p *Process) Logger() logger.Logger {
	return defaultLogger
}

type value struct {
	pos uintptr
	len int
}

func (val *value) Write(v []byte) int {
	if len(v) > val.len {
		return -1
	}
	for i, b := range v {
		*(*byte)(unsafe.Pointer(val.pos + uintptr(i)*unsafe.Sizeof(byte(0)))) = b
	}
	return len(v)
}

func (val *value) Read() []byte {
	return C.GoBytes(unsafe.Pointer(val.pos), C.int(val.len))
}

func (val *value) Len() int {
	return val.len
}

func NewReader(pos uintptr, len int) contract.Reader {
	return &value{pos: pos, len: len}
}

func NewWriter(pos uintptr, len int) contract.Writer {
	return &value{pos: pos, len: len}
}
