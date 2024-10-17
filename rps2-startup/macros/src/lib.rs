use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
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

    if f.sig.asyncness.is_some()
        || f.sig.unsafety.is_some()
        || f.sig.abi.is_some()
        || !f.sig.generics.params.is_empty()
        || f.sig.generics.where_clause.is_some()
        || !f.sig.inputs.is_empty()
        || f.sig.variadic.is_some()
    {
        return syn::Error::new_spanned(
            f.into_token_stream(),
            "`#[entry]` functions must have signature `fn() [-> impl Termination]`",
        )
        .to_compile_error()
        .into();
    }

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
