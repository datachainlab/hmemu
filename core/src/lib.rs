pub mod process;
pub mod types;
pub use process::*;
// // extern crate hmemu_codegen;
// // pub use hmemu_codegen::*;

// #[macro_export(local_inner_macros)]
// macro_rules! register {
//     ($($t:tt)*) => {{
//         struct Wrap;
//         impl Wrap {
//             register_impl!($($t)*);
//         }
//         Wrap::output()
//     }}
// }
