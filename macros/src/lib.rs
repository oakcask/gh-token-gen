
use proc_macro::Literal;
use proc_macro::TokenStream;
use syn::parse_macro_input;
use syn::LitStr;

#[proc_macro]
pub fn input_var(input: TokenStream) -> TokenStream {
    let lit = parse_macro_input!(input as LitStr);
    let name = lit.value();
    let var_name = format!("INPUT_{}", name.to_uppercase());
    let var_name = var_name.replace(" ", "_");
    Literal::string(&var_name).to_string().parse().unwrap()
}

#[proc_macro]
pub fn input_var_underscore(input: TokenStream) -> TokenStream {
    let lit = parse_macro_input!(input as LitStr);
    let name = lit.value();
    let var_name = format!("INPUT_{}", name.to_uppercase());
    let var_name = var_name.replace(" ", "_").replace("-", "_");
    Literal::string(&var_name).to_string().parse().unwrap()
}

#[proc_macro]
pub fn state_var(input: TokenStream) -> TokenStream {
    let lit = parse_macro_input!(input as LitStr);
    let name = lit.value();
    let var_name = format!("STATE_{}", name);
    Literal::string(&var_name).to_string().parse().unwrap()
}
