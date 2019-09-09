extern crate core;
pub use core::process;
pub use core::types;
pub use core::*;

extern crate hmemu_codegen;
pub use hmemu_codegen::*;

#[macro_export(local_inner_macros)]
macro_rules! contract_fn {
    ($($t:tt)*) => {{
        struct Wrap;
        impl Wrap {
            lookup_contract_fn_impl!($($t)*);
        }
        Wrap::output()
    }}
}
