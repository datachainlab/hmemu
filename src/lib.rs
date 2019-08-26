use std::cell::RefCell;
use std::collections::HashMap;
use std::panic::{catch_unwind, resume_unwind, AssertUnwindSafe, UnwindSafe};

#[link(name = "hm", kind = "dylib")]
extern "C" {
    fn __init_process() -> i32;
    fn __destroy_process() -> i32;
    fn __init_contract_address(value_ptr: *const u8, value_len: usize) -> i32;
    fn __init_sender(value_ptr: *const u8, value_len: usize) -> i32;
    fn __init_push_arg(value_ptr: *const u8, value_len: usize) -> i32;
    fn __init_done() -> i32;
    fn __clear() -> i32;

    fn __commit_state() -> i32;

    fn __get_mutex(pid: i32) -> i32;
    fn __release_mutex() -> i32;

    fn __get_return_value(offset: usize, value_buf_ptr: *mut u8, value_buf_len: usize) -> i32;
    fn __get_event(
        name: *const u8,
        name_len: usize,
        idx: usize,
        offset: usize,
        value_buf_ptr: *mut u8,
        value_buf_len: usize,
    ) -> i32;

    fn __push_contract_state(addr_ptr: *const u8, addr_len: usize) -> i32;
    fn __pop_contract_state() -> i32;
}

pub type Result<T> = std::result::Result<T, String>;

thread_local!(static PID: RefCell<i32> = RefCell::new(-1));

// get_mutex gets mutex
fn get_mutex() -> Result<()> {
    unsafe {
        match __get_mutex(get_pid()) {
            ret if ret < 0 => Err(format!("__get_mutex: error({})", ret)),
            _ => Ok(()),
        }
    }
}

// release_mutex release mutex
fn release_mutex() -> Result<()> {
    unsafe {
        match __release_mutex() {
            ret if ret < 0 => Err(format!("__release_mutex: error({})", ret)),
            _ => Ok(()),
        }
    }
}

fn get_pid() -> i32 {
    PID.with(|p| *p.borrow())
}

// init_process create a new process.
pub fn init_process() -> Result<i32> {
    match get_pid() {
        -1 => {}
        _ => return Err("process already exists".to_string()),
    }
    unsafe {
        match __init_process() {
            ret if ret < 0 => Err(format!("__init_process: error({})", ret)),
            pid => {
                PID.with(|p| {
                    *p.borrow_mut() = pid;
                });
                Ok(pid)
            }
        }
    }
}

pub fn init_contract_address(addr: &[u8]) -> Result<()> {
    unsafe {
        match __init_contract_address(addr.as_ptr(), addr.len()) {
            ret if ret < 0 => Err(format!("__init_contract_address: error({})", ret)),
            _ => Ok(()),
        }
    }
}

pub fn init_sender(addr: &[u8]) -> Result<()> {
    unsafe {
        match __init_sender(addr.as_ptr(), addr.len()) {
            ret if ret < 0 => Err(format!("__init_sender: error({})", ret)),
            _ => Ok(()),
        }
    }
}

pub fn init_push_arg<T: Into<String>>(s: T) -> Result<()> {
    let ss = s.into();
    let b = ss.as_bytes();
    unsafe {
        match __init_push_arg(b.as_ptr(), b.len()) {
            ret if ret < 0 => Err(format!("__init_push_arg: error({})", ret)),
            _ => Ok(()),
        }
    }
}

pub fn init_done() -> Result<()> {
    unsafe {
        match __init_done() {
            ret if ret < 0 => Err(format!("__init_done: error({})", ret)),
            _ => Ok(()),
        }
    }
}

pub fn clear() -> Result<()> {
    unsafe {
        match __clear() {
            ret if ret < 0 => Err(format!("__clear: error({})", ret)),
            _ => Ok(()),
        }
    }
}

pub fn commit_state() -> Result<()> {
    unsafe {
        match __commit_state() {
            ret if ret < 0 => Err(format!("__commit_state: error({})", ret)),
            _ => Ok(()),
        }
    }
}

const BUF_SIZE: usize = 128;

pub fn get_return_value() -> Result<Vec<u8>> {
    let mut buf = [0u8; BUF_SIZE];
    let mut offset = 0;
    let mut val: Vec<u8> = Vec::new();
    loop {
        match unsafe { __get_return_value(offset, buf.as_mut_ptr(), buf.len()) } {
            -1 => return Err("read_state: key not found".to_string()),
            0 => break,
            n => {
                val.extend_from_slice(&buf[0..n as usize]);
                if n < BUF_SIZE as i32 {
                    break;
                }
                offset += n as usize;
            }
        }
    }
    Ok(val)
}

pub fn get_event(name: &str, idx: usize) -> Result<Vec<u8>> {
    let mut buf = [0u8; BUF_SIZE];
    let mut offset = 0;
    let mut val: Vec<u8> = Vec::new();
    loop {
        match unsafe {
            __get_event(
                name.as_ptr(),
                name.len(),
                idx,
                offset,
                buf.as_mut_ptr(),
                buf.len(),
            )
        } {
            -1 => return Err("get_event: event not found".to_string()),
            0 => break,
            n => {
                val.extend_from_slice(&buf[0..n as usize]);
                if n < BUF_SIZE as i32 {
                    break;
                }
                offset += n as usize;
            }
        }
    }
    Ok(val)
}

pub fn destroy_process() -> Result<()> {
    unsafe {
        match __destroy_process() {
            ret if ret < 0 => Err(format!("__destroy_process: error({})", ret)),
            _ => PID.with(|p| {
                *p.borrow_mut() = -1;
                Ok(())
            }),
        }
    }
}

pub fn exec_process<T, F: FnOnce() -> Result<T>>(cb: F) -> Result<T>
where
    F: UnwindSafe,
{
    exec_process_with_arguments(Vec::<String>::new(), cb)
}

pub fn exec_process_with_arguments<T1, T2: Into<String>, F: FnOnce() -> Result<T1>>(
    args: Vec<T2>,
    cb: F,
) -> Result<T1>
where
    T2: UnwindSafe,
    F: UnwindSafe,
{
    exec_process_with_sender_and_arguments(&[], args, cb)
}

pub fn exec_process_with_sender<T, F: FnOnce() -> Result<T>>(sender: &[u8], cb: F) -> Result<T>
where
    F: UnwindSafe,
{
    exec_process_with_sender_and_arguments(sender, Vec::<String>::new(), cb)
}

pub fn exec_process_with_sender_and_arguments<T1, T2: Into<String>, F: FnOnce() -> Result<T1>>(
    sender: &[u8],
    args: Vec<T2>,
    cb: F,
) -> Result<T1>
where
    T2: UnwindSafe,
    F: UnwindSafe,
{
    run_process(|| call_contract(sender, args, cb))
}

pub fn exec_function<T, F: FnOnce() -> Result<T>>(f: F) -> Result<T>
where
    F: UnwindSafe,
{
    get_mutex()?;
    let mut res: Result<T> = Err(String::new());
    let result = {
        let mut resref = AssertUnwindSafe(&mut res);
        catch_unwind(move || {
            **resref = f();
        })
    };
    release_mutex()?;
    if let Err(err) = result {
        resume_unwind(err)
    } else {
        res
    }
}

pub fn run_process<T, F: FnOnce() -> Result<T>>(f: F) -> Result<T>
where
    F: UnwindSafe,
{
    if get_pid() >= 0 {
        Err("process already exists".to_string())
    } else {
        exec_function(|| {
            init_process()?;
            let res = f();
            destroy_process()?;
            res
        })
    }
}

pub fn call_contract<T1, T2: Into<String>, F: FnOnce() -> Result<T1>>(
    sender: &[u8],
    args: Vec<T2>,
    cb: F,
) -> Result<T1> {
    init_sender(sender)?;
    for arg in args.into_iter() {
        let s = arg.into();
        init_push_arg(s.as_str())?;
    }
    let res = match cb() {
        Ok(v) => {
            commit_state()?;
            Ok(v)
        }
        e => e,
    };
    clear()?;
    res
}

type Address = [u8; 20];
type ContractFn = fn() -> i32;

thread_local!(static VALUE_TABLE: RefCell<Vec<Vec<u8>>> = RefCell::new(Vec::new()));
thread_local!(static FUNC_TABLE: RefCell<HashMap<(Address, String), ContractFn >> = RefCell::new(HashMap::new()));

pub fn register_contract_function(addr: Address, name: String, f: ContractFn) {
    FUNC_TABLE.with(|t| {
        t.borrow_mut().insert((addr, name), f);
    });
}

#[no_mangle]
pub fn __read(id: usize, offset: usize, value_buf_ptr: *mut u8, value_buf_len: usize) -> i32 {
    VALUE_TABLE.with(|t| {
        let v = &t.borrow()[id];
        if v.len() > value_buf_len {
            return -1;
        }
        // TODO add support for offset
        if offset != 0 {
            panic!("offset option is unsupported")
        }
        let mut size = 0;
        let mut ptr = value_buf_ptr;
        for b in v {
            unsafe {
                *ptr = *b;
                ptr = ptr.wrapping_add(1);
            }
            size += 1;
        }
        size
    })
}

pub fn __write(v: Vec<u8>) -> usize {
    VALUE_TABLE.with(|t| {
        let mut vv = t.borrow_mut();
        vv.push(v);
        vv.len() - 1
    })
}

#[no_mangle]
pub fn __call_contract(
    addr_ptr: *const u8,
    addr_size: usize,
    entry_ptr: *const u8,
    entry_size: usize,
    args: *const u8,
    args_size: usize,
) -> i32 {
    let mut a_ptr = addr_ptr;
    let mut e_ptr = entry_ptr;

    let mut addr: Address = Default::default();
    for i in 0..addr_size {
        unsafe {
            addr[i] = *a_ptr;
        }
        a_ptr = a_ptr.wrapping_add(1);
    }
    let mut entry = vec![];
    for _ in 0..entry_size {
        unsafe {
            entry.push(*e_ptr);
        }
        e_ptr = e_ptr.wrapping_add(1);
    }
    let entry_name = String::from_utf8(entry).unwrap();

    FUNC_TABLE.with(|t| {
        match t.borrow().get(&(addr, entry_name)) {
            Some(f) => {
                unsafe {
                    // TODO should we pass arguments via this step?
                    __push_contract_state(addr_ptr, addr_size);
                }
                match f() {
                    _ => {
                        let res = get_return_value().unwrap();
                        let id = __write(res) as i32;
                        unsafe {
                            __pop_contract_state();
                        }
                        id
                    }
                }
            }
            None => {
                panic!("function not found");
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initialize_test() {
        exec_function(|| {
            init_process().unwrap();
            init_push_arg("key1").unwrap();
            init_done().unwrap();

            init_push_arg("key2").expect_err("expect error");
            destroy_process().unwrap();
            Ok(())
        })
        .unwrap();
    }

    #[test]
    fn process_test() {
        for i in 0..10 {
            let args = vec!["1".to_string(), i.to_string()];
            exec_process_with_arguments(args, || {
                let x = hmc::get_arg_str(0).unwrap().parse::<i64>().unwrap();
                let y = hmc::get_arg_str(1).unwrap().parse::<i64>().unwrap();
                assert_eq!(1 + i, x + y);
                Ok(())
            })
            .unwrap();
        }
    }

    #[test]
    fn sender_test() {
        let sender = b"d11234567890ABCDEFFF";
        exec_process_with_sender(sender, || {
            let s = hmc::get_sender().unwrap();
            assert_eq!(&s, sender);
            Ok(())
        })
        .unwrap();

        for i in 0..10 {
            let args = vec!["1".to_string(), i.to_string()];
            exec_process_with_sender_and_arguments(sender, args, || {
                let s = hmc::get_sender().unwrap();
                assert_eq!(&s, sender);

                let x = hmc::get_arg_str(0).unwrap().parse::<i64>().unwrap();
                let y = hmc::get_arg_str(1).unwrap().parse::<i64>().unwrap();
                assert_eq!(1 + i, x + y);

                Ok(())
            })
            .unwrap();
        }
    }

    #[test]
    fn state_test() {
        exec_process(|| {
            let key = "key".as_bytes();
            let value = "value".as_bytes();

            hmc::write_state(key, value);
            hmc::read_state(key).expect_err("expect error");

            Ok(())
        })
        .unwrap();
    }

    #[test]
    fn process_manager_test() {
        let pid1 = exec_function(|| {
            let pid = init_process().unwrap();
            assert_eq!(pid, get_pid());
            init_push_arg("key1").unwrap();
            Ok(pid)
        })
        .unwrap();
        let th = std::thread::spawn(move || {
            exec_function(|| {
                let pid = init_process().unwrap();
                assert_eq!(pid, get_pid());
                assert_ne!(pid, pid1);
                init_push_arg("key2").unwrap();
                init_done().unwrap();
                assert_eq!("key2", hmc::get_arg_str(0).unwrap().as_str());
                Ok(())
            })
            .unwrap();
        });
        th.join().unwrap();
        exec_function(|| {
            assert_eq!(pid1, get_pid());
            assert_eq!("key1", hmc::get_arg_str(0).unwrap().as_str());
            Ok(())
        })
        .unwrap();
    }

    #[test]
    fn exec_function_assert_test() {
        catch_unwind(|| {
            exec_function(|| {
                assert_eq!(true, false);
                Ok(())
            })
            .unwrap();
        })
        .unwrap_err();
        exec_function(|| Ok(())).unwrap();
    }

    #[test]
    fn call_contract_test() {
        let sender1 = b"00000000000000000001";
        let sender2 = b"00000000000000000002";
        let key = "key".as_bytes();
        let value = "value".as_bytes();
        run_process(|| {
            {
                let args = vec!["1", "2", "3"];
                call_contract(sender1, args.clone(), || {
                    let sender = hmc::get_sender().unwrap();
                    assert_eq!(sender1, &sender);
                    for i in 0..args.len() {
                        let arg = hmc::get_arg_str(i)?;
                        assert_eq!(args[i], arg);
                    }
                    hmc::write_state(key, value);
                    Ok(0)
                })?;
            }

            {
                let args = vec!["4", "5"];
                call_contract(sender2, args.clone(), || {
                    let sender = hmc::get_sender().unwrap();
                    assert_eq!(sender2, &sender);
                    for i in 0..args.len() {
                        let arg = hmc::get_arg_str(i)?;
                        assert_eq!(args[i], arg);
                    }
                    let v = hmc::read_state(key)?;
                    assert_eq!(value, (&v as &[u8]));
                    Ok(0)
                })
            }
        })
        .unwrap();
    }

    #[test]
    fn run_process_test() {
        let key = "key".as_bytes();
        let value = "value".as_bytes();

        // simple test
        run_process(|| {
            hmc::write_state(key, value);
            Ok(0)
        })
        .unwrap();

        // nested runners
        run_process(|| {
            hmc::write_state(key, value);
            assert!(run_process(|| {
                hmc::write_state(key, value);
                Ok(0)
            })
            .is_err());
            Ok(0)
        })
        .unwrap();
    }

    #[test]
    fn call_external_contract_test() {
        const SENDER: Address = *b"00000000000000000001";
        const CONTRACT_A: Address = *b"00000000000000000010";
        const CONTRACT_B: Address = *b"00000000000000000011";
        const CONTRACT_C: Address = *b"00000000000000000012";

        // 1. call external contract simply, and ensure returned value matches expected
        {
            fn func_a() -> i32 {
                let external_contract = hmc::get_arg(0).unwrap();
                let res =
                    hmc::call_contract(&external_contract, "func_b".as_bytes(), vec![]).unwrap();
                hmc::return_value(format!("got {}", String::from_utf8(res).unwrap()).as_bytes())
            }
            fn func_b() -> i32 {
                hmc::return_value("ok".as_bytes())
            }

            run_process(|| {
                init_contract_address(&CONTRACT_A)?;
                register_contract_function(CONTRACT_B, "func_b".to_string(), func_b);

                call_contract(
                    &SENDER,
                    vec![String::from_utf8(CONTRACT_B.to_vec()).unwrap()],
                    || {
                        let s = hmc::get_sender().unwrap();
                        assert_eq!(SENDER, s);
                        func_a();
                        Ok(0)
                    },
                )?;

                let ret = get_return_value()?;
                assert_eq!("got ok".to_string().into_bytes(), ret);

                Ok(())
            })
            .unwrap();
        }

        // 2. ensure caller address of external contract matches each contract address or sender
        {
            fn func_a() -> i32 {
                let external_contract = hmc::get_arg(0).unwrap();
                assert_eq!(SENDER, hmc::get_sender().unwrap());
                let res =
                    hmc::call_contract(&external_contract, "func_b".as_bytes(), vec![]).unwrap();
                assert_eq!(SENDER, hmc::get_sender().unwrap());
                hmc::return_value(&res)
            }
            fn func_b() -> i32 {
                assert_eq!(CONTRACT_A, hmc::get_sender().unwrap());
                // TODO contract address is given via argument
                let res = hmc::call_contract(&CONTRACT_C, "func_c".as_bytes(), vec![]).unwrap();
                assert_eq!(CONTRACT_A, hmc::get_sender().unwrap());
                hmc::return_value(&res)
            }
            fn func_c() -> i32 {
                assert_eq!(CONTRACT_B, hmc::get_sender().unwrap());
                hmc::return_value(&hmc::get_sender().unwrap())
            }

            run_process(|| {
                init_contract_address(&CONTRACT_A)?;
                register_contract_function(CONTRACT_B, "func_b".to_string(), func_b);
                register_contract_function(CONTRACT_C, "func_c".to_string(), func_c);

                call_contract(
                    &SENDER,
                    vec![String::from_utf8(CONTRACT_B.to_vec()).unwrap()],
                    || {
                        let s = hmc::get_sender().unwrap();
                        assert_eq!(SENDER, s);
                        func_a();
                        Ok(0)
                    },
                )?;

                let ret = get_return_value()?;
                assert_eq!(ret, CONTRACT_B);

                Ok(())
            })
            .unwrap();
        }

        // 3. ensure each updated contract state is valid
        {
            fn func_a() -> i32 {
                let key = "key_a".as_bytes();
                let value = "value_a".as_bytes();

                let external_contract = hmc::get_arg(0).unwrap();
                hmc::write_state(key, value);
                let res =
                    hmc::call_contract(&external_contract, "func_b".as_bytes(), vec![]).unwrap();
                hmc::return_value(format!("got {}", String::from_utf8(res).unwrap()).as_bytes())
            }
            fn func_b() -> i32 {
                let key = "key_b".as_bytes();
                let value = "value_b".as_bytes();

                let res = hmc::call_contract(&CONTRACT_C, "func_c".as_bytes(), vec![]).unwrap();
                hmc::write_state(key, value);
                hmc::return_value(format!("got {}", String::from_utf8(res).unwrap()).as_bytes())
            }
            fn func_c() -> i32 {
                let key = "key_c".as_bytes();
                let value = "value_c".as_bytes();

                hmc::write_state(key, value);
                hmc::return_value("ok".as_bytes())
            }

            run_process(|| {
                init_contract_address(&CONTRACT_A)?;
                register_contract_function(CONTRACT_B, "func_b".to_string(), func_b);
                register_contract_function(CONTRACT_C, "func_c".to_string(), func_c);

                call_contract(
                    &SENDER,
                    vec![String::from_utf8(CONTRACT_B.to_vec()).unwrap()],
                    || {
                        let s = hmc::get_sender().unwrap();
                        assert_eq!(SENDER, s);
                        func_a();
                        Ok(0)
                    },
                )?;

                let ret = get_return_value()?;
                assert_eq!("got got ok".to_string().into_bytes(), ret);
                commit_state()?;

                assert_eq!("value_a", hmc::read_state_str("key_a".as_bytes()).unwrap());
                assert!(hmc::read_state_str("key_b".as_bytes()).is_err());

                init_contract_address(&CONTRACT_B)?;
                assert_eq!("value_b", hmc::read_state_str("key_b".as_bytes()).unwrap());
                assert!(hmc::read_state_str("key_c".as_bytes()).is_err());

                init_contract_address(&CONTRACT_C)?;
                assert_eq!("value_c", hmc::read_state_str("key_c".as_bytes()).unwrap());

                Ok(())
            })
            .unwrap();
        }
    }
}
