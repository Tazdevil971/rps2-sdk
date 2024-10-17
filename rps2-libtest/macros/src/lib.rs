use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use syn::parse::ParseStream;
use syn::{parse_macro_input, AttrStyle, Attribute, Expr, ItemFn, Lit, LitStr, Meta, Path, Token};

struct Ignore {
    reason: Option<String>,
}

struct ShouldPanic {
    #[allow(unused)]
    expected: Option<String>,
}

struct TestAttrs {
    should_panic: Option<ShouldPanic>,
    ignore: Option<Ignore>,
}

#[proc_macro_attribute]
pub fn test(args: TokenStream, input: TokenStream) -> TokenStream {
    if !args.is_empty() {
        return syn::Error::new_spanned(
            TokenStream2::from(args),
            "this attribute accepts no arguments",
        )
        .to_compile_error()
        .into();
    }

    let mut f = parse_macro_input!(input as ItemFn);

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
            "`#[test]` functions must have signature `fn() [-> impl Termination]`",
        )
        .to_compile_error()
        .into();
    }

    let test_attrs = match extract_attrs(&mut f.attrs) {
        Ok(res) => res,
        Err(err) => return err.to_compile_error().into(),
    };

    let cfgs = get_cfgs(&f.attrs);
    let name = f.sig.ident.to_string();
    let func = &f.sig.ident;

    let mut test = quote! {
        __rps2_libtest::TestBuilder::new(
            ::core::concat!(::core::module_path!(), "::", #name),
            || __rps2_libtest::__hidden::test_invoke(#func)
        )
    };

    if let Some(ignore) = test_attrs.ignore {
        if let Some(reason) = ignore.reason {
            test = quote! { #test.with_ignore_reason(#reason) };
        } else {
            test = quote! { #test.with_ignore() }
        }
    }

    if test_attrs.should_panic.is_some() {
        test = quote! { #test.with_should_panic() };
    }

    test = quote! { #test.build() };

    quote!(
        #(#cfgs)*
        const _: () = {
            extern crate rps2_libtest as __rps2_libtest;
            __rps2_libtest::__hidden::inventory::submit! { #test }
        };

        #f
    )
    .into()
}

fn extract_attrs(attrs: &mut Vec<Attribute>) -> syn::Result<TestAttrs> {
    // Extract #[test] specific attributes
    let mut ignore = None;
    let mut should_panic = None;

    attrs.retain(|attr| {
        if attr.path().is_ident("ignore") {
            ignore = Some(attr.clone());
            false
        } else if attr.path().is_ident("should_panic") {
            should_panic = Some(attr.clone());
            false
        } else {
            true
        }
    });

    Ok(TestAttrs {
        should_panic: should_panic.map(parse_should_panic).transpose()?,
        ignore: ignore.map(parse_ignore).transpose()?,
    })
}

fn parse_ignore(attr: Attribute) -> syn::Result<Ignore> {
    assert!(attr.path().is_ident("ignore"));

    match &attr.meta {
        Meta::Path(_) => return Ok(Ignore { reason: None }),
        Meta::NameValue(meta) => {
            if let Expr::Lit(expr) = &meta.value {
                if let Lit::Str(lit) = &expr.lit {
                    return Ok(Ignore {
                        reason: Some(lit.value()),
                    });
                }
            }
        }
        Meta::List(_) => {}
    }

    Err(syn::Error::new_spanned(
        attr,
        "attribute must be of the form `#[ignore]` or `#[ignore = \"reason\"]`",
    ))
}

fn parse_should_panic(attr: Attribute) -> syn::Result<ShouldPanic> {
    assert!(attr.path().is_ident("should_panic"));

    match &attr.meta {
        Meta::Path(_) => return Ok(ShouldPanic { expected: None }),
        Meta::NameValue(meta) => {
            if let Expr::Lit(expr) = &meta.value {
                if let Lit::Str(lit) = &expr.lit {
                    return Ok(ShouldPanic {
                        expected: Some(lit.value()),
                    });
                }
            }
        }
        Meta::List(meta) => {
            if let Ok(res) = meta.parse_args_with(|parser: ParseStream| {
                let path = parser.parse::<Path>()?;
                if !path.is_ident("expected") {
                    // This error doesn't really matter, it will be overwritten anyway
                    return Err(syn::Error::new_spanned(&attr, "..."));
                }

                let _equal = parser.parse::<Token![=]>()?;
                let lit = parser.parse::<LitStr>()?;
                let _comma = parser.parse::<Option<Token![,]>>()?;

                Ok(ShouldPanic {
                    expected: Some(lit.value()),
                })
            }) {
                return Ok(res);
            }
        }
    }

    Err(syn::Error::new_spanned(
        &attr,
        "attribute must be of the form `#[should_panic]`, `#[should_panic = \"expected\"]` or `#[should_panic(expected = \"expected\")]`"
    ))
}

fn get_cfgs(attrs: &[Attribute]) -> Vec<&Attribute> {
    attrs
        .into_iter()
        .filter(|attr| attr.style == AttrStyle::Outer && attr.path().is_ident("cfg"))
        .collect()
}
