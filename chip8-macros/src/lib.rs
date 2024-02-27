use proc_macro::TokenStream;
use quote::quote;
use syn::{parse::Parse, parse_macro_input, Arm, Expr, Lit, Pat};
struct OperationMatches {
    opcode: u16,
    wildcard_mask: u16,
    operation: Box<Expr>,
}
impl OperationMatches {
    pub fn new(hex: String, func: Box<Expr>) -> Self {
        let mut match_operation = 0;
        for c in hex.chars() {
            match_operation *= 16;
            if let Some(digit) = c.to_digit(16) {
                match_operation += digit as u16;
            }
        }
        let mut match_mask = 0;
        for (i,c) in hex.chars().rev().enumerate() {
            if !matches!(c, 'x' | 'y' | 'k' | 'n') {
                match_mask += 0b1111 << i * 4
            }
        }
        Self {
            opcode: match_operation,
            wildcard_mask: match_mask,
            operation: func,
        }
    }
    pub fn generate(&self, operation: &Expr) -> proc_macro2::TokenStream {
        let mask = self.wildcard_mask;
        let oper = self.opcode;
        let func = &self.operation;
        quote! {
            if (#operation & #mask == #oper) {
                #func;
                return;
            }
        }
    }
}
struct OperationsHolder {
    to_match: Expr,
    arms: Vec<OperationMatches>,
}
impl Parse for OperationsHolder {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let opcode = input.parse()?;
        let mut match_arms = vec![];
        
        while let Ok(arm) = input.parse::<Arm>() {
            if let Pat::Lit(lit) = arm.pat {
                if let Lit::Str(hex) = lit.lit {
                    match_arms.push(OperationMatches::new(hex.value(), arm.body))
                }
            }
        }

        Ok(Self {
            to_match: opcode,
            arms: match_arms
        })
    }
}
impl OperationsHolder {
    pub fn generate(&self) -> proc_macro2::TokenStream {
        let op = &self.to_match;
        let arms = self.arms
            .iter()
            .map(|c| c.generate(op))
            .collect::<Vec<proc_macro2::TokenStream>>();
        quote! {
            #(#arms)*
        }
    }
}
#[proc_macro]
pub fn opcode_handler(input: TokenStream) -> TokenStream {
    let operations = parse_macro_input!(input as OperationsHolder);
    operations.generate().into()
}
