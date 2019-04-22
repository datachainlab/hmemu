extern crate hmc;
extern crate hmemu;

pub fn main() {
    hmemu::init_process();
    hmemu::init_push_arg("1");
    hmemu::init_push_arg("2");
    hmemu::init_done();

    let x = hmc::get_arg_str(0).unwrap().parse::<i64>().unwrap();
    let y = hmc::get_arg_str(1).unwrap().parse::<i64>().unwrap();

    let ret = format!("{}", x+y);
    hmc::log(ret.as_bytes());

    let key = "key";
    hmc::write_state(key.as_bytes(), format!("value").as_bytes());
    hmemu::commit_state();
    match hmc::read_state(key.as_bytes()) {
        Ok(v) => {
            println!("Ok: {}", String::from_utf8(v.to_vec()).unwrap());
        }
        Err(e) => {
            panic!(e);
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
        let x = hmc::get_arg_str(0).unwrap().parse::<i64>().unwrap();
        let y = hmc::get_arg_str(1).unwrap().parse::<i64>().unwrap();
        assert_eq!(4, x + y);
    }
}
