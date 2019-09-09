extern crate hmcdk;
use hmcdk::api;
use hmcdk::prelude::*;

#[contract]
pub fn init() -> R<i32> {
    Ok(None)
}

#[contract]
pub fn contract_func() -> R<i64> {
    let x: i64 = api::get_arg(0)?;
    let y: i64 = api::get_arg(1)?;

    api::log(&format!("{}", x + y).to_bytes());

    let key = "key";
    api::write_state(key.as_bytes(), format!("value").as_bytes());
    api::emit_event("test-event", key.as_bytes())?;
    Ok(Some(x + y))
}

mod othermod {
    use super::*;

    #[contract]
    pub fn cfn() -> R<String> {
        Ok(Some("ok".to_string()))
    }
}

#[cfg(test)]
mod tests {
    extern crate hmemu;
    use super::*;
    use hmcdk::error;
    use hmcdk::utils;
    use hmemu::contract_fn;
    use hmemu::types::ArgsBuilder;

    #[test]
    fn simple_process_execution() {
        let mut args_ = ArgsBuilder::new();
        args_.push(1i64);
        args_.push(2i64);

        hmemu::exec_process_with_arguments(args_.convert_to_vec(), || {
            let x: i64 = api::get_arg(0)?;
            let y: i64 = api::get_arg(1)?;
            assert_eq!(3, x + y);

            Ok(())
        })
        .unwrap();
    }

    #[test]
    fn contract_func_test() {
        let mut args_ = ArgsBuilder::new();
        args_.push(1i64);
        args_.push(2i64);
        hmemu::exec_process_with_arguments(args_.convert_to_vec(), || {
            let ret = contract_func()?;
            hmemu::commit_state()?;

            let state = api::read_state::<String>("key".as_bytes())?;
            assert_eq!("value", state.as_str());

            assert_eq!(Some(3), ret);

            let ev = hmemu::get_event("test-event", 0);
            assert!(ev.is_ok());
            assert_eq!("key".to_string(), String::from_utf8(ev.unwrap()).unwrap());

            Ok(())
        })
        .unwrap();
    }

    #[contract]
    fn external_func() -> R<i32> {
        Ok(Some(100))
    }

    fn hex_to_address(hex_str: &str) -> Result<Address, error::Error> {
        let b = utils::hex_to_bytes(hex_str);
        let mut addr: Address = Default::default();
        addr.copy_from_slice(&b);
        if addr.len() == 20 {
            Ok(addr)
        } else {
            Err(error::from_str("invalid length"))
        }
    }

    #[test]
    fn call_test() {
        let sender = hex_to_address("0x1221a0726d56aedea9dbe2522ddae3dd8ed0f36c").unwrap();
        let contract = hex_to_address("0xd8eba1f372b9e0d378259f150d52c2e6c2e4109a").unwrap();
        hmemu::run_process(|| {
            hmemu::register_contract_function(
                contract,
                "get_balance".to_string(),
                contract_fn!(external_func),
            );

            hmemu::call_contract(&sender, ArgsBuilder::new().convert_to_vec(), || {
                let ret: i32 =
                    api::call_contract(&contract, "get_balance".as_bytes(), vec![]).unwrap();
                assert_eq!(100, ret);
                Ok(())
            })?;
            Ok(())
        })
        .unwrap();
    }

    #[test]
    fn lookup_contract_fn_test() {
        let sender = hex_to_address("0x1221a0726d56aedea9dbe2522ddae3dd8ed0f36c").unwrap();
        hmemu::run_process(|| {
            hmemu::call_contract(&sender, ArgsBuilder::new().convert_to_vec(), || {
                assert_eq!(contract_fn!(othermod::cfn)(), 0);
                Ok(())
            })?;
            Ok(())
        })
        .unwrap();
    }
}
