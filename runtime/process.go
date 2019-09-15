package main

import "C"
import (
	"errors"
	"fmt"
	"sync"
	"unsafe"
	"container/list"

	"github.com/bluele/hypermint/pkg/abci/store"
	sdk "github.com/bluele/hypermint/pkg/abci/types"
	"github.com/bluele/hypermint/pkg/contract"
	"github.com/bluele/hypermint/pkg/contract/event"
	"github.com/bluele/hypermint/pkg/db"
	"github.com/bluele/hypermint/pkg/logger"
	"github.com/ethereum/go-ethereum/common"
	dbm "github.com/tendermint/tm-db"
)

var (
	defaultLogger = logger.GetDefaultLogger("*:debug").With("module", "process")
)

type ProcessManager struct {
	mu   sync.Mutex
	cpid int
	pss  []*Process
}

func (pm *ProcessManager) GetMutex(pid int) {
	pm.mu.Lock()
	if pid >= 0 {
		pm.cpid = pid
	}
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

var _ contract.Process = (*Process)(nil)

type Process struct {
	initialized bool
	kvs sdk.KVStore
	db          *db.VersionedDB

	contractAddress common.Address
	sender common.Address
	args   contract.Args
	res    []byte

	stateStack *list.List
	sets db.RWSets
	entries []*event.Entry
}

func NewProcess() (*Process, error) {
	kvs, err := newKVS()
	if err != nil {
		return nil, err
	}
	return &Process{kvs: kvs, db: db.NewVersionedDB(kvs.Prefix(common.Address{}.Bytes())), stateStack: list.New()}, nil
}

func newKVS() (sdk.KVStore, error) {
	mdb := dbm.NewMemDB()
	cms := store.NewCommitMultiStore(mdb)
	var key = sdk.NewKVStoreKey("main")
	cms.MountStoreWithDB(key, sdk.StoreTypeIAVL, nil)
	if err := cms.LoadLatestVersion(); err != nil {
		return nil, err
	}
	return cms.GetKVStore(key), nil
}

func (p *Process) Sender() common.Address {
	return p.sender
}

func (p *Process) ContractAddress() common.Address {
	return p.contractAddress
}

func (p *Process) Args() contract.Args {
	return p.args
}

func (p *Process) GetArg(idx int) ([]byte, error) {
	arg, ok := p.args.Get(idx)
	if !ok {
		return nil, contract.ErrArgIdxNotFound
	}
	return arg, nil
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

func (p *Process) EmitEvent(e *event.Entry) {
	p.entries = append(p.entries, e)
}

// TODO this method should be moved into NewProcess?
func (p *Process) InitContractAddress(addr common.Address) {
	copy(p.contractAddress[:], addr[:])
	p.db = db.NewVersionedDB(p.kvs.Prefix(p.contractAddress.Bytes()))
}

func (p *Process) PushState(contractAddressBytes contract.Reader) {
	var nextContract common.Address
	copy(nextContract[:], contractAddressBytes.Read())
	p.stateStack.PushFront(Process{
		initialized: p.initialized,
		contractAddress: p.contractAddress,
		sender: p.sender,
		args: p.args,
		res: p.res,
		db: p.db,
	})
	// clear
	p.initialized = false
	p.sender = p.contractAddress
	p.contractAddress = nextContract
	p.args = contract.Args{}
	p.res = nil
	p.db = db.NewVersionedDB(p.kvs.Prefix(nextContract.Bytes()))
}

func (p *Process) PopState() {
	if p.stateStack.Len() < 1 {
		panic("stack is empty")
	}
	elem := p.stateStack.Front()
	top := elem.Value.(Process)
	p.initialized = top.initialized
	p.sender = top.sender
	p.args = top.args
	p.res = top.res
	p.sets = append(p.sets, &db.RWSet{
		Address: p.contractAddress,
		Items:   p.db.RWSetItems(),
	})
	p.contractAddress = top.contractAddress
	p.db = top.db
	p.stateStack.Remove(elem)
}

func (p *Process) CommitState() error {
	sets := make([]*db.RWSet, len(p.sets))
	copy(sets[:], p.sets)
	set := &db.RWSet{
		Address: p.contractAddress,
		Items:   p.db.RWSetItems(),
	}
	sets = append(sets, set)
	return db.CommitState(p.kvs, sets, db.Version{1, 1}, db.NewKeyMaps())
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
	if val.len == 0 {
		return []byte{}
	}
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
