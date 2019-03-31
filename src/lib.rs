#![recursion_limit = "128"]

extern crate proc_macro;

mod deserialize;
mod serialize;

use crate::proc_macro::TokenStream;
use proc_macro2::TokenStream as QuoteOutput;
use quote::quote;
use syn::{parse, Data, DataEnum, DeriveInput, Expr, ExprLit, Lit};

#[proc_macro_derive(DeserializeEnum)]
pub fn derive_deserialize_enum(input: TokenStream) -> TokenStream {
    let syntax_tree = parse(input).unwrap();

    deserialize::derive(syntax_tree)
}

#[proc_macro_derive(SerializeEnum)]
pub fn derive_serialize_enum(input: TokenStream) -> TokenStream {
    let syntax_tree = parse(input).unwrap();

    serialize::derive(syntax_tree)
}

fn get_enum_data(input: &DeriveInput) -> &DataEnum {
    match input.data {
        Data::Enum(ref data) => data,
        _ => panic!("Can't derive DeserializeEnum on types that aren't enums"),
    }
}

fn collect_variant_ids(data: &DataEnum) -> Vec<QuoteOutput> {
    data.variants
        .iter()
        .scan(0_i64, |index, variant| match &variant.discriminant {
            Some((
                _,
                Expr::Lit(ExprLit {
                    lit: Lit::Int(value),
                    ..
                }),
            )) => {
                *index = value.value() as i64 + 1;
                Some(quote! { #value })
            }
            Some(_) => panic!("Expecting enum with integer discriminants"),
            None => {
                let id = *index;

                *index += 1;

                Some(quote! { #id })
            }
        })
        .collect()
}
