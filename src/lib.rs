//! Attribute macro for turning free functions into extension traits.
//!
//! ```
//! # #[cfg_attr(not(unix), ignore)]
//! # {
//! use methodify::methodify;
//! use std::path::Path;
//! use std::os::unix::fs::PermissionsExt;
//!
//! #[methodify]
//! fn is_executable<P: AsRef<Path>>(path: &P) -> bool {
//!     let path = path.as_ref();
//!
//!     path.metadata().is_ok_and(|metadata| {
//!         metadata.is_file() && metadata.permissions().mode() & 0o111 != 0
//!     })
//! }
//!
//! let executable = std::env::current_exe().unwrap();
//! assert!(executable.is_executable());
//! # }
//! ```

use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{
    Error, FnArg, Ident, ItemFn, Pat, PatIdent, Receiver, Result, Type, parse_macro_input,
    parse_quote,
};

#[proc_macro_attribute]
pub fn methodify(args: TokenStream, input: TokenStream) -> TokenStream {
    if !args.is_empty() {
        return Error::new(
            Span::call_site(),
            "`#[methodify]` does not accept arguments",
        )
        .into_compile_error()
        .into();
    }

    let function = parse_macro_input!(input as ItemFn);

    expand(function)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}

fn expand(function: ItemFn) -> Result<TokenStream2> {
    if function.sig.asyncness.is_some() {
        return Err(Error::new_spanned(
            function.sig.asyncness,
            "`#[methodify]` does not currently support async functions",
        ));
    }

    if function.sig.constness.is_some() {
        return Err(Error::new_spanned(
            function.sig.constness,
            "`#[methodify]` does not support const functions",
        ));
    }

    let mut method_sig = function.sig.clone();
    let first_arg = method_sig.inputs.first_mut().ok_or_else(|| {
        Error::new_spanned(
            &method_sig.ident,
            "`#[methodify]` requires at least one function argument",
        )
    })?;

    let receiver = receiver_for(first_arg)?;
    let impl_type = impl_type_for(first_arg)?;
    *first_arg = FnArg::Receiver(receiver);
    method_sig.generics = Default::default();

    let first_arg_name = first_arg_name(function.sig.inputs.first().expect("checked above"))?;
    let function_name = &function.sig.ident;
    let trait_name = trait_name_for(function_name);
    let visibility = &function.vis;
    let trait_generics = &function.sig.generics;
    let (impl_generics, trait_type_generics, where_clause) = function.sig.generics.split_for_impl();
    let remaining_args = function.sig.inputs.iter().skip(1).map(argument_expression);

    Ok(quote! {
        #visibility trait #trait_name #trait_generics {
            #method_sig;
        }

        impl #impl_generics #trait_name #trait_type_generics for #impl_type #where_clause {
            #method_sig {
                #function_name(#first_arg_name, #(#remaining_args),*)
            }
        }

        #function
    })
}

fn receiver_for(first_arg: &FnArg) -> Result<Receiver> {
    let FnArg::Typed(arg) = first_arg else {
        return Err(Error::new_spanned(
            first_arg,
            "`#[methodify]` expects the first argument to be a normal function parameter",
        ));
    };

    match arg.ty.as_ref() {
        Type::Reference(reference) if reference.mutability.is_some() => Ok(parse_quote!(&mut self)),
        Type::Reference(_) => Ok(parse_quote!(&self)),
        _ => Ok(parse_quote!(self)),
    }
}

fn impl_type_for(first_arg: &FnArg) -> Result<Type> {
    let FnArg::Typed(arg) = first_arg else {
        return Err(Error::new_spanned(first_arg, "expected a typed argument"));
    };

    match arg.ty.as_ref() {
        Type::Reference(reference) => Ok((*reference.elem).clone()),
        ty => Ok(ty.clone()),
    }
}

fn first_arg_name(first_arg: &FnArg) -> Result<TokenStream2> {
    let FnArg::Typed(arg) = first_arg else {
        return Err(Error::new_spanned(first_arg, "expected a typed argument"));
    };

    match arg.ty.as_ref() {
        Type::Reference(reference) if reference.mutability.is_some() => Ok(quote!(self)),
        Type::Reference(_) => Ok(quote!(self)),
        _ => Ok(quote!(self)),
    }
}

fn argument_expression(arg: &FnArg) -> TokenStream2 {
    match arg {
        FnArg::Typed(arg) => match arg.pat.as_ref() {
            Pat::Ident(PatIdent { ident, .. }) => quote!(#ident),
            pat => quote!(#pat),
        },
        FnArg::Receiver(_) => quote!(self),
    }
}

fn trait_name_for(function_name: &Ident) -> Ident {
    let name = function_name.to_string();
    let mut output = String::new();
    let mut uppercase_next = true;

    for ch in name.chars() {
        if ch == '_' {
            uppercase_next = true;
        } else if uppercase_next {
            output.extend(ch.to_uppercase());
            uppercase_next = false;
        } else {
            output.push(ch);
        }
    }

    Ident::new(&output, Span::call_site())
}
