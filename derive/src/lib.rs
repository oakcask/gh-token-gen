use proc_macro::TokenStream;
use syn::{DataStruct, DeriveInput, Error, Fields, parse_macro_input, spanned::Spanned};
use quote::quote;

use crate::{codegen::{InputField, OutputField}, parse::{InputAttr, InputSource, OutputAttr}};
mod codegen;
mod parse;

#[proc_macro_derive(ActionInput, attributes(input))]
pub fn derive_input(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    match input.data {
        syn::Data::Struct(DataStruct {
            fields: Fields::Named(fields),
            ..
        }) => {
            let mut inputs = Vec::with_capacity(fields.named.len());
            for f in fields.named.into_iter() {
                let span = f.span();
                if let Some(ident) = f.ident {
                    let attrs = match InputAttr::parse_attributes(&f.attrs) {
                        Ok(a) => a,
                        Err(e) => return compile_error(e).into()
                    };
                    inputs.push(InputField {
                        span,
                        field: ident,
                        attrs
                    });
                }
            }
            let struct_name = input.ident;
            codegen::action_input_impl(struct_name, inputs).unwrap_or_else(|e| compile_error(e)).into()
        },
        _ => compile_error(Error::new(input.span(), "`#[derive(ActionInput)]` only supports non-tuple structs")).into()
    }
}

#[proc_macro_derive(ActionOutput, attributes(output))]
pub fn derive_output(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    match input.data {
        syn::Data::Struct(DataStruct {
            fields: Fields::Named(fields),
            ..
        }) => {
            let mut inputs = Vec::with_capacity(fields.named.len());
            for f in fields.named.into_iter() {
                if let Some(ident) = f.ident {
                    let attrs = match OutputAttr::parse_attributes(&f.attrs) {
                        Ok(a) => a,
                        Err(e) => return compile_error(e).into()
                    };
                    inputs.push(OutputField {
                        field: ident,
                        attrs
                    });
                }
            }
            let struct_name = input.ident;
            codegen::action_output_impl(struct_name, inputs).unwrap_or_else(|e| compile_error(e)).into()
        },
        _ => compile_error(Error::new(input.span(), "`#[derive(ActionOutput)]` only supports non-tuple structs")).into()
    }
}

#[proc_macro_attribute]
pub fn wasm_action(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let mut output = input.clone();
    let start_fn = codegen::start_fn(parse_macro_input!(input)).unwrap_or_else(|e| compile_error(e));
    let start_fn: TokenStream = start_fn.into();
    output.extend(start_fn);
    output
}

fn compile_error(error: syn::Error) -> proc_macro2::TokenStream {
    let err = error.to_compile_error();
    quote!(
        #err
    )
}
