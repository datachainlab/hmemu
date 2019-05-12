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

pub type Result<T> = std::result::Result<T, String>;

thread_local!(static PID: RefCell<i32> = RefCell::new(-1));

// get_mutex gets mutex
fn get_mutex() -> Result<()> {
    unsafe {
        match __get_mutex() {
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

// This function gets a mutex inside.
// init_process create a new process.
pub fn init_process() -> Result<i32> {
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
            _ => Ok(())
        }
    }
}

pub fn init_push_arg<T: Into<String>>(s: T) -> Result<()> {
    let ss = s.into();
    let b = ss.as_bytes();
    unsafe {
        match __init_push_arg(b.as_ptr(), b.len()) {
            ret if ret < 0 => Err(format!("__init_push_arg: error({})", ret)),
            _ => Ok(())
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

pub fn switch_process(pid: i32) -> Result<()> {
    unsafe {
        match __switch_process(pid) {
            ret if ret < 0 => Err(format!("__switch_process: error({})", ret)),
            _ => PID.with(|p| {
                *p.borrow_mut() = pid;
                Ok(())
            }),
        }
    }
}

pub fn exec_process<F: FnOnce() -> Result<()>>(cb: F) -> Result<()> {
    exec_process_with_arguments(Vec::<String>::new(), cb)
}

pub fn exec_process_with_arguments<F: FnOnce() -> Result<()>, T: Into<String>>(
    args: Vec<T>,
    cb: F,
) -> Result<()> {
    get_mutex()?;
    init_process()?;
    for arg in args.into_iter() {
        let s = arg.into();
        init_push_arg(s.as_str())?;
    }
    init_done()?;
    let res = cb();
    destroy_process()?;
    release_mutex()?;
    res
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initialize_test() {
        init_process().unwrap();
        init_push_arg("key1").unwrap();
        init_done().unwrap();

        init_push_arg("key2").expect_err("expect error");
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
        get_mutex().unwrap();

        let pid1 = init_process().unwrap();
        assert_eq!(pid1, get_pid());
        init_push_arg("key1").unwrap();

        let pid2 = init_process().unwrap();
        assert_eq!(pid2, get_pid());
        init_push_arg("key2").unwrap();

        switch_process(pid1).unwrap();
        assert_eq!(pid1, get_pid());
        assert_eq!("key1", hmc::get_arg_str(0).unwrap().as_str());

        switch_process(pid2).unwrap();
        assert_eq!(pid2, get_pid());
        assert_eq!("key2", hmc::get_arg_str(0).unwrap().as_str());

        release_mutex().unwrap();
    }
}
