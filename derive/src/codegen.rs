use proc_macro2::{Span, TokenStream};
use syn::{DeriveInput, Error, Ident, LitStr, Type, token::Token};
use quote::{TokenStreamExt, quote};

use crate::{InputAttr, InputSource, parse::{AttrValue, InputAttrKey, OutputAttr, OutputName}};

pub(crate) fn start_fn(input: DeriveInput) -> Result<TokenStream, Error> {
    let ident = input.ident;
    Ok(quote!{
        #[wasm_bindgen::prelude::wasm_bindgen]
        pub async fn start() -> Result<(), wasm_bindgen::prelude::JsError> {
            use builder::Action;
            let input = #ident::parse_input()?;
            if let Some(state) = #ident::parse_state()? {
              Ok(#ident::post(input, state).await?)
            } else {
              let output = #ident::main(input).await?;
              use builder::ActionOutput;
              output.save().await?;
              Ok(())
            }
        }        
    })
}

#[derive(Debug)]
pub(crate) struct InputField {
    pub(crate) span: Span,
    pub(crate) field: Ident,
    pub(crate) ty: Type,
    pub(crate) attrs: Vec<InputAttr>,
}

impl InputField {
    pub(crate) fn input_source<'a>(&'a self) -> Result<InputSource<'a>, Error> {
        InputSource::try_from(self.span, &self.attrs)
    }
}

pub(crate) fn action_input_impl(struct_name: Ident, fields: Vec<InputField>) -> Result<TokenStream, Error> {
    let mut ts = TokenStream::new();
    for f in fields {
        let tokens = action_input_field_init(f)?;
        ts.append_all(tokens);
    }
    let out = quote! {
    impl builder::ActionInput for #struct_name {
        fn parse() -> Result<Self, wasm_actions_core::error::Error> {
            Ok(Self {
                #ts
            })
        }
      }   
    };
    Ok(out)
}

fn action_input_field_init(field: InputField) -> Result<TokenStream, Error> {
    let source = field.input_source()?;
    let field = field.field.clone();
    let code = match source {
    InputSource::Input(name) => {
        quote! {
            #field : (builder::ParseInput::parse(wasm_actions::get_input!(#name).ok_or_else(|| wasm_actions_core::error::Error::from(std::format!("{} missing", #name)))?)?),
        }
    },
    InputSource::Env(env) => {
        quote! {
            #field : (builder::ParseInput::parse(wasm_actions::env::var(#env).ok_or_else(|| wasm_actions_core::error::Error::from(std::format!("${} missing", #env)))?)?),
        }
    },
    InputSource::InputThenEnv { input: name, env } => {
        quote! {
            #field : (builder::ParseInput::parse(
                wasm_actions::get_input!(#name).or_else(|_|
                    wasm_actions::env::var(#env)
                ).ok_or_else(|| wasm_actions_core::error::Error::from("either {} or ${} missing"))?
            )?),
        }
    },
  };
  Ok(code)
}

#[derive(Debug)]
pub(crate) struct OutputField {
    pub(crate) span: Span,
    pub(crate) field: Ident,
    pub(crate) attrs: Vec<OutputAttr>,
}

pub(crate) fn action_output_impl(struct_name: Ident, fields: Vec<OutputField>) -> Result<TokenStream, Error> {
    let mut ts = TokenStream::new();
    for f in fields {
        if let Some(tokens) = action_output_set_output(f)? {
            ts.append_all(tokens);
        }
    }

    let code = quote! {
        impl builder::ActionOutput for #struct_name {
            fn parse() -> Result<Option<Self>, wasm_actions_core::error::Error> {
                if let Some(state) = wasm_actions::get_state!("wasm_actions") {
                    Ok(Some(serde_json::from_str(&state).map_err(Error::new)?))
                } else {
                    Ok(None)
                }
            }

            async fn save(self) -> Result<(), wasm_actions_core::error::Error> {
                let json = serde_json::to_string(&self).map_err(wasm_actions_core::error::Error::new)?;
                wasm_actions::save_state("wasm_actions", &json).await?;
                #ts
                Ok(())
            }
        }   
    };
    Ok(code)
}

fn action_output_set_output(field: OutputField) -> Result<Option<TokenStream>, Error> {
    if let Some(name) = OutputName::try_from(field.span, &field.attrs) {
        let struct_field = field.field;
        Ok(Some(quote! {
            let #struct_field = builder::StringifyOutput::stringify(self.#struct_field);
            wasm_actions::set_output(#name, &#struct_field).await?;
        }))
    } else {
        Ok(None)
    }
}
