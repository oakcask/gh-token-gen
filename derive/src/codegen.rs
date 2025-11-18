use proc_macro2::TokenStream;
use syn::{DeriveInput, Error, Ident, Type, token::Token};
use quote::quote;

use crate::{InputAttr, parse::{AttrValue, InputAttrKey}};

pub(crate) fn main_fn(input: DeriveInput) -> Result<TokenStream, Error> {
    let ident = input.ident;
    Ok(quote!{
        #[allow(dead_code)]
        #[wasm_bindgen::prelude::wasm_bindgen]
        pub async fn main() -> Result<(), wasm_bindgen::prelude::JsError> {
            let input = #ident::<builder::Action>::parse_input()?;
            if let Some(state) = #ident::<builder::Action>::parse_state()? {
              Ok(#ident::<builder::Action>::post(input, state).await?)
            } else {
              let output = #ident::<builder::Action>::main(input).await?;
              output.save()?;
              Ok(())
            }
        }        
    })
}

#[derive(Debug)]
pub(crate) struct InputField {
    pub(crate) field: Ident,
    pub(crate) ty: syn::Type,
    pub(crate) attrs: Vec<InputAttr>
}

enum InputStrategy<'a> {
    Input(&'a AttrValue),
    Env(&'a AttrValue),
    InputThenEnv { input: &'a AttrValue, env: &'a AttrValue }
}

pub(crate) fn action_input_impl(struct_name: Ident, fields: Vec<InputField>) -> Result<TokenStream, Error> {

  
    let out = quote! {
    impl builder::ActionInput for #struct_name {
        fn parse() -> Result<Self, wasm_actions_core::error::Error> {
            Ok(Self {

            })
        }
    }   
    };

  eprintln!("{out}");
  Ok(out)
}

//       #field1 : wasm_actions::get_input!(#name).ok_or_else(|| Error::from("{} missing", #name))?.try_into().map_err(|e| Error::new(e))?,
//       #field2 : wasm_actions::env::var(#env).unwrap_or_else(|| Error::from("${} missing", #env))?.try_into().map_err(|e| Error::new(e))?,
fn action_input_field_init(field: InputField) -> Result<TokenStream, Error> {
  let name = field.field;
  match field.
  Ok(quote! {
    #name : ,
  })
}