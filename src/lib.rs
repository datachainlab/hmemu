#[link(name = "hm", kind = "dylib")]
extern "C" {
    fn __init_process() -> i32;
    fn __destroy_process() -> i32;
    fn __init_sender(value_ptr: *const u8, value_len: usize) -> i32;
    fn __init_push_arg(value_ptr: *const u8, value_len: usize) -> i32;
    fn __init_done() -> i32;
    fn __commit_state() -> i32;
}

pub fn init_process() {
    unsafe {
        match __init_process() {
            -1 => panic!("abort"),
            _ => {}
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
            _ => {}
        }
    }
}

pub fn exec_process<F: FnOnce() -> Result<(), String>>(cb: F) -> Result<(), String> {
    exec_process_with_arguments(Vec::<String>::new(), cb)
}

pub fn exec_process_with_arguments<F: FnOnce() -> Result<(), String>, T: Into<String>>(args: Vec<T>, cb: F) -> Result<(), String> {
    init_process();
    for arg in args.into_iter() {
        let s = arg.into();
        init_push_arg(s.as_str());
    }
    init_done();
    let res = cb();
    destroy_process();
    res
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn process_test() {
        for _ in 0..10 {
            init_process();
            init_push_arg("1");
            init_push_arg("2");
            init_done();

            let x = hmc::get_arg_str(0).unwrap().parse::<i64>().unwrap();
            let y = hmc::get_arg_str(1).unwrap().parse::<i64>().unwrap();
            assert_eq!(3, x + y);

            destroy_process();
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
        }).unwrap();
    }
}
