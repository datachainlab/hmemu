extern crate proc_macro;
use crate::proc_macro::TokenStream;
use quote::quote;
use syn;
use syn::{parse_macro_input, FnArg};

#[proc_macro]
pub fn lookup_contract_fn_impl(input: TokenStream) -> TokenStream {
    let item = process_input(input.clone());
    let ast = parse_macro_input!(item as FnArg);

    let q = quote! {
        #[inline]
        fn output() -> fn() -> i32 {
            #ast
        }
    };
    let t: TokenStream = q.into();
    t
}

fn process_input(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as FnArg);
    let path = match ast {
        syn::FnArg::Ignored(syn::Type::Path(mut x)) => {
            let id = x.path.segments.len();
            let mut seg = &mut x.path.segments[id - 1];
            // convert into contract API
            let ident = format!("__{}", seg.ident);
            seg.ident = syn::Ident::new(&ident, seg.ident.span());
            x
        }
        _ => panic!("failed!"),
    };
    let gen = quote! {
        #path
    };
    let t: TokenStream = gen.into();
    t
}
