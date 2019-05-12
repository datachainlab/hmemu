use std::cell::RefCell;

#[link(name = "hm", kind = "dylib")]
extern "C" {
    fn __init_process() -> i32;
    fn __destroy_process() -> i32;
    fn __init_sender(value_ptr: *const u8, value_len: usize) -> i32;
    fn __init_push_arg(value_ptr: *const u8, value_len: usize) -> i32;
    fn __init_done() -> i32;
    fn __commit_state() -> i32;

    fn __get_mutex(pid: i32) -> i32;
    fn __release_mutex() -> i32;
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

#[allow(dead_code)]
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

pub fn commit_state() -> Result<()> {
    unsafe {
        match __commit_state() {
            ret if ret < 0 => Err(format!("__commit_state: error({})", ret)),
            _ => Ok(()),
        }
    }
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

pub fn exec_process<T, F: FnOnce() -> Result<T>>(cb: F) -> Result<T> {
    exec_process_with_arguments(Vec::<String>::new(), cb)
}

pub fn exec_process_with_arguments<T1, T2: Into<String>, F: FnOnce() -> Result<T1>>(
    args: Vec<T2>,
    cb: F,
) -> Result<T1> {
    exec_function(|| {
        init_process()?;
        for arg in args.into_iter() {
            let s = arg.into();
            init_push_arg(s.as_str())?;
        }
        init_done()?;
        let res = cb();
        destroy_process()?;
        res
    })
}

pub fn exec_function<T, F: FnOnce() -> Result<T>>(f: F) -> Result<T> {
    get_mutex()?;
    let res = f();
    release_mutex()?;
    res
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
}
