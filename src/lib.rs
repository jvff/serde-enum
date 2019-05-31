#![recursion_limit = "128"]

extern crate proc_macro;

mod deserialize;
mod serialize;
mod variant_id;

use crate::{proc_macro::TokenStream, variant_id::VariantId};
use proc_macro2::TokenStream as QuoteOutput;
use quote::ToTokens;
use syn::{parse, Data, DataEnum, DeriveInput, Expr, ExprLit, Lit, Variant};

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
        .scan(VariantId::default(), |variant_id, variant| {
            let new_variant_id =
                retrieve_discriminant_id(variant).unwrap_or_else(|| variant_id.clone());

            *variant_id = new_variant_id.next();
            Some(new_variant_id.into_token_stream())
        })
        .collect()
}

fn retrieve_discriminant_id(variant: &Variant) -> Option<VariantId> {
    variant
        .discriminant
        .as_ref()
        .map(|discriminant| match discriminant {
            (
                _,
                Expr::Lit(ExprLit {
                    lit: Lit::Int(value),
                    ..
                }),
            ) => VariantId::from_value(value.value() as u32),
            _ => panic!("Expecting enum with integer discriminants"),
        })
}
