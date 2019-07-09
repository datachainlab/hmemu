extern crate hmc;
#[cfg(test)]
extern crate hmemu;

pub fn init() {}

pub fn exec() -> Result<i32, String> {
    let x = hmc::get_arg_str(0).unwrap().parse::<i64>().unwrap();
    let y = hmc::get_arg_str(1).unwrap().parse::<i64>().unwrap();

    let ret = format!("{}", x + y);
    hmc::log(ret.as_bytes());

    let key = "key";
    hmc::write_state(key.as_bytes(), format!("value").as_bytes());
    hmc::emit_event("test-event", key.as_bytes())?;
    hmc::return_value("ok".as_bytes());
    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let args = vec!["1", "2"];
        hmemu::exec_process_with_arguments(args, || {
            assert_eq!(2 + 2, 4);
            let x = hmc::get_arg_str(0).unwrap().parse::<i64>().unwrap();
            let y = hmc::get_arg_str(1).unwrap().parse::<i64>().unwrap();
            assert_eq!(3, x + y);

            Ok(())
        })
        .unwrap();
    }

    #[test]
    fn exec_test() {
        let args = vec!["1", "2"];
        hmemu::exec_process_with_arguments(args, || {
            exec().unwrap();
            hmemu::commit_state()?;

            let state = hmc::read_state("key".as_bytes());
            assert!(state.is_ok());
            assert_eq!("value".as_bytes().to_vec(), state.unwrap());

            let ret = hmemu::get_return_value();
            assert!(ret.is_ok());
            assert_eq!("ok".to_string(), String::from_utf8(ret.unwrap()).unwrap());

            let ev = hmemu::get_event("test-event", 0);
            assert!(ev.is_ok());
            assert_eq!("key".to_string(), String::from_utf8(ev.unwrap()).unwrap());

            Ok(())
        })
        .unwrap();
    }
}
