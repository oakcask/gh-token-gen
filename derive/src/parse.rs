use std::process::Output;

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::ToTokens;
use syn::{Attribute, Error, Expr, Field, FieldsNamed, Ident, LitStr, Token, parse::Parse, punctuated::Punctuated, spanned::Spanned};

pub(crate) enum InputSource<'a> {
    Input(&'a AttrValue),
    Env(&'a AttrValue),    
    InputThenEnv {
        input: &'a AttrValue,
        env: &'a AttrValue
    }
}

impl<'a> InputSource<'a> {
    pub(crate) fn try_from(span: Span, attrs: &'a [InputAttr]) -> Result<Self, syn::Error> {
        let mut input = None;
        let mut env = None;
        for a in attrs {
            match a.key {
                InputAttrKey::Name => input = Some(&a.value),

                InputAttrKey::Env => env = Some(&a.value),
                _ => {},
            }
        }

        match (input, env) {
            (None, None) => Err(Error::new(span, "#[input] expectes at least one name or env")),
            (None, Some(e)) => Ok(Self::Env(e)),
            (Some(i), None) => Ok(Self::Input(i)),
            (Some(i), Some(e)) => Ok(Self::InputThenEnv { input: i, env: e }),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum InputAttrKey {
    Name,
    Env,
    Required,
    Description,
    Default
}

#[derive(Debug)]
pub(crate) struct InputAttr {
    pub(crate) key: InputAttrKey,
    pub(crate) value: AttrValue,
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum OutputAttrKey {
    Name,
    Description,
}

#[derive(Debug)]
pub(crate) struct OutputAttr {
    pub(crate) key: OutputAttrKey,
    pub(crate) value: AttrValue,
}

#[derive(Debug)]
pub(crate) enum AttrValue {
    LitStr(LitStr),
    Expr(Expr)
}

impl ToTokens for AttrValue {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            AttrValue::LitStr(lit_str) => lit_str.to_tokens(tokens),
            AttrValue::Expr(expr) => expr.to_tokens(tokens),
        }
    }
}

impl InputAttr {
    pub(crate) fn parse_attributes(attrs: &[Attribute]) -> Result<Vec<Self>, syn::Error> {
        let mut input_attrs = Vec::new();

        for a in attrs {
            if a.path().is_ident("input") {
                let kvs = a.parse_args_with(Punctuated::<InputAttr, Token![,]>::parse_terminated)?;
                for kv in kvs {
                    input_attrs.push(kv)
                }
            }
        }

        Ok(input_attrs)
    }
}

impl Parse for InputAttr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let key: Ident = input.parse()?;
        let key_s = key.to_string();
        let key = match key_s.as_str() {
            "default" => InputAttrKey::Default,
            "description" => InputAttrKey::Description,
            "env" => InputAttrKey::Env,
            "name" => InputAttrKey::Name,
            "required" => InputAttrKey::Required,
            unknown => return Err(Error::new(key.span(), format!("#[input] cannot accept `{unknown}`")))
        };
        let t_assign = input.parse::<Token![=]>()?;
        let value = if input.peek(LitStr) {
            let lit: LitStr = input.parse()?;
            AttrValue::LitStr(lit)
        } else {
            match input.parse::<Expr>() {
                Ok(expr) => AttrValue::Expr(expr),
                Err(_) => return Err(Error::new(t_assign.span, "expected literal string or expression after `=`")),
            }
        };
        Ok(Self { key, value })
    }
}

impl OutputAttr {
    pub(crate) fn parse_attributes(attrs: &[Attribute]) -> Result<Vec<Self>, syn::Error> {
        let mut input_attrs = Vec::new();

        for a in attrs {
            if a.path().is_ident("output") {
                let kvs = a.parse_args_with(Punctuated::<OutputAttr, Token![,]>::parse_terminated)?;
                for kv in kvs {
                    input_attrs.push(kv)
                }
            }
        }

        Ok(input_attrs)
    }
}

impl Parse for OutputAttr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let key: Ident = input.parse()?;
        let key_s = key.to_string();
        let key = match key_s.as_str() {
            "description" => OutputAttrKey::Description,
            "name" => OutputAttrKey::Name,
            unknown => return Err(Error::new(key.span(), format!("#[output] cannot accept `{unknown}`")))
        };
        let t_assign = input.parse::<Token![=]>()?;
        let value = if input.peek(LitStr) {
            let lit: LitStr = input.parse()?;
            AttrValue::LitStr(lit)
        } else {
            match input.parse::<Expr>() {
                Ok(expr) => AttrValue::Expr(expr),
                Err(_) => return Err(Error::new(t_assign.span, "expected literal string or expression after `=`")),
            }
        };
        Ok(Self { key, value })
    }
}
