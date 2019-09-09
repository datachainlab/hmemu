# hmemu

An emulation library to ease contract development and testing for hypermint.

This version supports hypermint **v0.4.2**.

## Getting started

First, please append following code to Cargo.toml in your project.

```toml
[dependencies]
hmcdk = { git = "https://github.com/bluele/hypermint", tag = "v0.4.2" }

[dev-dependencies]
hmemu = { git = "https://github.com/bluele/hmemu", branch = "develop" }
```

Then, you write a test code for contract.

```rust
extern crate hmemu;

#[test]
fn contract_func_test() {
    let args = vec!["1", "2"];
    hmemu::exec_process_with_arguments(args, || {
        contract_func().unwrap();
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
```

Finally, you can run test command.

```
// This is required to build `lib` directory. Please see `build.rs` for details.
$ export GO111MODULE=on
$ cargo test
```

## Test

```
$ export GO111MODULE=on
$ cargo test
```

## Author

**Jun Kimura**

* <http://github.com/bluele>
* <junkxdev@gmail.com>
