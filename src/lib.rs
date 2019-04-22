#[link(name="hm", kind="dylib")]
extern "C" {
    fn __init_process() -> i64;
    fn __init_sender(value_ptr: *const u8, value_len: usize) -> i64;
    fn __init_push_arg(value_ptr: *const u8, value_len: usize) -> i64;
    fn __init_done() -> i64;
    fn __commit_state() -> i64;
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
    let b =  ss.as_bytes();
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
