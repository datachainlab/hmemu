extern crate hmc;
extern crate hmemu;

pub fn main() {
    match exec() {
        Err(e) => panic!(e),
        _ => {}
    }
}

pub fn exec() -> Result<i32, String> {
    hmemu::init_process()?;
    hmemu::init_push_arg("1")?;
    hmemu::init_push_arg("2")?;
    hmemu::init_done()?;

    let x = hmc::get_arg_str(0).unwrap().parse::<i64>().unwrap();
    let y = hmc::get_arg_str(1).unwrap().parse::<i64>().unwrap();

    let ret = format!("{}", x+y);
    hmc::log(ret.as_bytes());

    let key = "key";
    hmc::write_state(key.as_bytes(), format!("value").as_bytes());
    hmc::emit_event("test-event", key.as_bytes()).unwrap();
    hmc::return_value("ok".as_bytes());
    hmemu::commit_state()?;
    match hmc::read_state(key.as_bytes()) {
        Ok(v) => {
            println!("Ok: {}", String::from_utf8(v.to_vec()).unwrap());
        }
        Err(e) => {
            panic!(e);
        }
    }
    match hmemu::get_return_value() {
        Ok(v) => println!("return: {}", String::from_utf8(v).unwrap()),
        Err(e) => panic!(e),
    }
    match hmemu::get_event("test-event", 0) {
        Ok(v) => println!("event: {}", String::from_utf8(v).unwrap()),
        Err(e) => panic!(e),
    }
    Ok(0)
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        hmemu::init_process();
        hmemu::init_push_arg("1");
        hmemu::init_push_arg("2");
        hmemu::init_done();

        assert_eq!(2 + 2, 4);
        let x = hmc::get_arg_str(0).unwrap().parse::<i64>().unwrap();
        let y = hmc::get_arg_str(1).unwrap().parse::<i64>().unwrap();
        assert_eq!(3, x + y);
    }
}
