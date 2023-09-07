use proc_macro::TokenStream;
use quote::{format_ident, quote, quote_spanned, ToTokens};
use syn::{
    parse_macro_input, parse_quote, spanned::Spanned, FnArg, ItemFn, Pat, PathArguments,
    ReturnType, Type, Visibility,
};

#[proc_macro_attribute]
pub fn stringify_err(_args: TokenStream, input: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(input as ItemFn);

    if match input.sig.output {
        ReturnType::Default => true,
        ReturnType::Type(_, ref ty) => match &**ty {
            Type::Path(p) => p
                .path
                .segments
                .last()
                .map_or(true, |s| !s.ident.to_string().contains("Result")),
            _ => true,
        },
    } {
        let span = if let ReturnType::Type(_, t) = &input.sig.output {
            t.span()
        } else {
            input.sig.span()
        };
        return quote_spanned! {span=>
            ::core::compile_error!("stringify_err currently only supports functions that return `Result`s");
        }
        .into();
    }

    let ok_type = match input.sig.output {
        ReturnType::Type(_, ref ty) => {
            match &**ty {
                Type::Path(p) => match &p.path.segments.first().unwrap().arguments {
                    PathArguments::AngleBracketed(a) => a.args.first().unwrap(),
                    _ => return quote_spanned! (p.span()=> ::core::compile_error!("stringify_err: `Result` that is the return must at least have the Ok type");).into(),
                },
                _ => unreachable!(),
            }
        }
        ReturnType::Default => unreachable!(),
    };

    let mut has_self_argument = false;
    // remove types from args for use when calling the inner function
    let mut args_without_types = vec![];
    let mut args_without_types_including_self = vec![];
    for arg in &input.sig.inputs {
        match arg {
            FnArg::Receiver(_) => {
                has_self_argument = true;
                args_without_types_including_self.push(quote!(self));
            }
            FnArg::Typed(arg) => {
                let tokens = if let Pat::Ident(mut a) = *arg.pat.clone() {
                    a.attrs.clear();
                    a.mutability = None;
                    a.into_token_stream()
                } else {
                    arg.pat.clone().into_token_stream()
                };
                args_without_types.push(tokens.clone());
                args_without_types_including_self.push(tokens);
            }
        }
    }

    let self_dot = if has_self_argument {
        quote!(self.)
    } else {
        quote!()
    };

    let asyncness_await = match input.sig.asyncness {
        Some(_) => quote!(.await),
        None => quote!(),
    };

    let attrs = input.attrs.clone();
    let vis = input.vis.clone();
    let mut sig = input.sig.clone();
    sig.output = parse_quote!(-> ::core::result::Result<#ok_type, ::std::string::String>);

    let orig_name = input.sig.ident.clone();
    let inner_name = format_ident!("_stringify_err_inner_{}", orig_name);

    input.sig.ident = inner_name.clone();
    input.vis = Visibility::Inherited; // make sure the inner function isn't leaked to the public
    input.attrs = vec![
        // we will put the original attributes on the function we make
        // we also don't want the inner function to appear in docs or autocomplete (if they do, they should be deprecated and give a warning if they are used)
        parse_quote!(#[doc(hidden)]),
        parse_quote!(#[deprecated = "inner function for stringify-err. Please do not use!"]),
        parse_quote!(#[inline(always)]), // let's make sure we don't produce more overhead than we need to, the output should produce similar assembly to the input (besides the end)
    ];

    // for functions that take a self argument, we will need to put the inner function outside of our new function since we don't know what type self is
    let (outer_input, inner_input) = if has_self_argument {
        (Some(input), None)
    } else {
        (None, Some(input))
    };

    quote! {
        #outer_input

        #(#attrs)* #vis #sig {
            #inner_input

            #[allow(deprecated)]
            match #self_dot #inner_name(#(#args_without_types),*) #asyncness_await {
                ::core::result::Result::Ok(ok) => ::core::result::Result::Ok(ok),
                // https://docs.rs/eyre/latest/eyre/struct.Report.html#display-representations
                ::core::result::Result::Err(err) => ::core::result::Result::Err(format!("{:#}", err)),
            }
        }
    }
    .into()
}
