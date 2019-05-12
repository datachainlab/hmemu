use std::cell::RefCell;

#[link(name = "hm", kind = "dylib")]
extern "C" {
    fn __init_process() -> i32;
    fn __destroy_process() -> i32;
    fn __init_sender(value_ptr: *const u8, value_len: usize) -> i32;
    fn __init_push_arg(value_ptr: *const u8, value_len: usize) -> i32;
    fn __init_done() -> i32;
    fn __commit_state() -> i32;

    fn __get_mutex() -> i32;
    fn __release_mutex() -> i32;
    fn __switch_process(pid: i32) -> i32;
}

thread_local!(static PID: RefCell<i32> = RefCell::new(-1));

// get_mutex gets mutex
fn get_mutex() {
    unsafe {
        match __get_mutex() {
            0 => {}
            _ => panic!("cannot get mutex"),
        }
    }
}

// release_mutex release mutex
fn release_mutex() {
    unsafe {
        match __release_mutex() {
            0 => {}
            _ => panic!("cannot release mutex"),
        }
    }
}

#[allow(dead_code)]
fn get_pid() -> i32 {
    PID.with(|p| *p.borrow())
}

// This function gets a mutex inside.
// init_process create a new process.
pub fn init_process() -> i32 {
    unsafe {
        match __init_process() {
            -1 => panic!("abort"),
            pid => {
                PID.with(|p| {
                    *p.borrow_mut() = pid;
                });
                pid
            }
        }
    }
}

pub fn init_sender(addr: &[u8]) {
    unsafe {
        match __init_sender(addr.as_ptr(), addr.len()) {
            -1 => panic!("abort"),
            _ => {}
        }
    }
}

pub fn init_push_arg<T: Into<String>>(s: T) {
    let ss = s.into();
    let b = ss.as_bytes();
    unsafe {
        match __init_push_arg(b.as_ptr(), b.len()) {
            -1 => panic!("abort"),
            _ => {}
        }
    }
}

pub fn init_done() {
    unsafe {
        match __init_done() {
            -1 => panic!("abort"),
            _ => {}
        }
    }
}

pub fn commit_state() {
    unsafe {
        match __commit_state() {
            -1 => panic!("abort"),
            _ => {}
        }
    }
}

pub fn destroy_process() {
    unsafe {
        match __destroy_process() {
            -1 => panic!("abort"),
            _ => PID.with(|p| {
                *p.borrow_mut() = -1;
            }),
        }
    }
}

pub fn switch_process(pid: i32) {
    unsafe {
        match __switch_process(pid) {
            -1 => panic!("abort"),
            _ => PID.with(|p| {
                *p.borrow_mut() = pid;
            }),
        }
    }
}

pub fn exec_process<F: FnOnce() -> Result<(), String>>(cb: F) -> Result<(), String> {
    exec_process_with_arguments(Vec::<String>::new(), cb)
}

pub fn exec_process_with_arguments<F: FnOnce() -> Result<(), String>, T: Into<String>>(
    args: Vec<T>,
    cb: F,
) -> Result<(), String> {
    get_mutex();
    init_process();
    for arg in args.into_iter() {
        let s = arg.into();
        init_push_arg(s.as_str());
    }
    init_done();
    let res = cb();
    destroy_process();
    release_mutex();
    res
}

#[cfg(test)]
mod tests {
    use super::*;

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
        get_mutex();

        let pid1 = init_process();
        assert_eq!(pid1, get_pid());
        init_push_arg("key1");

        let pid2 = init_process();
        assert_eq!(pid2, get_pid());
        init_push_arg("key2");

        switch_process(pid1);
        assert_eq!(pid1, get_pid());
        assert_eq!("key1", hmc::get_arg_str(0).unwrap().as_str());

        switch_process(pid2);
        assert_eq!(pid2, get_pid());
        assert_eq!("key2", hmc::get_arg_str(0).unwrap().as_str());

        release_mutex();
    }
}
