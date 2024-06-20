use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse_macro_input, AttrStyle, Attribute, ItemFn};

#[proc_macro_attribute]
pub fn entry(args: TokenStream, input: TokenStream) -> TokenStream {
    if !args.is_empty() {
        return syn::Error::new_spanned(
            TokenStream2::from(args),
            "this attribute accepts no arguments",
        )
        .to_compile_error()
        .into();
    }

    let f = parse_macro_input!(input as ItemFn);
    let name = f.sig.ident.clone();
    let cfgs = get_cfgs(&f.attrs);

    quote!(
        #(#cfgs)*
        const _: () = {
            extern crate rps2_startup as __rps2_startup;

            #[no_mangle]
            extern "Rust" fn __stage3_entry() -> i32 {
                __rps2_startup::__hidden::stage3_invoke(#name)
            }
        };

        #f
    )
    .into()
}

fn get_cfgs(attrs: &[Attribute]) -> Vec<&Attribute> {
    attrs
        .into_iter()
        .filter(|attr| attr.style == AttrStyle::Outer && attr.path().is_ident("cfg"))
        .collect()
}
