#![recursion_limit = "128"]

extern crate proc_macro;

mod deserialize;
mod serialize;
mod variant_id;

use crate::{proc_macro::TokenStream, variant_id::VariantId};
use proc_macro2::{Span, TokenStream as QuoteOutput};
use quote::ToTokens;
use syn::{
    parse,
    parse::{ParseStream, Parser},
    spanned::Spanned,
    Data, DataEnum, DeriveInput, Expr, ExprLit, ExprParen, Ident, Lit, Path, Variant,
};

#[proc_macro_derive(DeserializeEnum, attributes(serde_enum))]
pub fn derive_deserialize_enum(input: TokenStream) -> TokenStream {
    let syntax_tree = parse(input).unwrap();

    deserialize::derive(syntax_tree)
}

#[proc_macro_derive(SerializeEnum, attributes(serde_enum))]
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
            let new_variant_id = retrieve_discriminant_id(variant)
                .or_else(|| retrieve_attribute_id(variant))
                .unwrap_or_else(|| variant_id.clone());

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

fn retrieve_attribute_id(variant: &Variant) -> Option<VariantId> {
    variant
        .attrs
        .iter()
        .filter(|attr| attr.path == Path::from(Ident::new("serde_enum", Span::call_site())))
        .filter_map(|attr| Parser::parse2(parse_attribute_id, attr.tts.clone()).ok())
        .next()
}

fn parse_attribute_id(input: ParseStream) -> Result<VariantId, syn::Error> {
    let expression: ExprParen = input.parse()?;
    let assignment = match *expression.expr {
        Expr::Assign(assignment) => assignment,
        other => {
            return Err(syn::Error::new(
                other.span(),
                "expected \"variant_id = ...\"",
            ))
        }
    };

    match *assignment.left {
        Expr::Path(ref expr)
            if expr.path == Path::from(Ident::new("variant_id", Span::call_site())) =>
        {
            Ok(VariantId::from_expression(
                assignment.right.into_token_stream(),
            ))
        }
        other => Err(syn::Error::new(other.span(), "expected \"variant_id\"")),
    }
}
