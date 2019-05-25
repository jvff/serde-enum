use crate::{collect_variant_ids, get_enum_data, proc_macro::TokenStream};
use proc_macro2::{Span, TokenStream as QuoteOutput};
use quote::quote;
use syn::{
    DataEnum, DeriveInput, Fields, GenericParam, Generics, Ident, Lifetime, LifetimeDef, LitStr,
    TypeGenerics,
};

pub fn derive(input: DeriveInput) -> TokenStream {
    let name = &input.ident;
    let name_str = LitStr::new(&name.to_string(), Span::call_site());
    let generics = build_generics(input.generics.clone());
    let (impl_generics, _, where_clause) = generics.split_for_impl();
    let (_, type_generics, _) = input.generics.split_for_impl();

    let data = get_enum_data(&input);
    let variants = retrieve_variant_names(&data);
    let visitor = build_visitor(name, &type_generics, &data);

    let output = quote! {
        impl #impl_generics ::serde::Deserialize<'de> for #name #type_generics #where_clause {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: ::serde::Deserializer<'de>,
            {
                const VARIANTS: &[&str] = & #variants;

                #visitor

                deserializer.deserialize_enum(#name_str, VARIANTS, Visitor)
            }
        }
    };

    output.into()
}

fn build_generics(mut generics: Generics) -> Generics {
    let de_lifetime = Lifetime::new("'de", Span::call_site());

    generics
        .params
        .insert(0, GenericParam::Lifetime(LifetimeDef::new(de_lifetime)));

    generics
}

fn retrieve_variant_names(data: &DataEnum) -> QuoteOutput {
    let names = data
        .variants
        .iter()
        .map(|variant| LitStr::new(&variant.ident.to_string(), Span::call_site()));

    quote! {
        [ #( #names ),* ]
    }
}

fn build_visitor(name: &Ident, generics: &TypeGenerics, data: &DataEnum) -> QuoteOutput {
    let expecting = LitStr::new(&format!("enum {}", name), Span::call_site());

    let variant_id = collect_variant_ids(data);
    let variant_deserialization = generate_variant_deserializations(name, data);

    quote! {
        struct Visitor;

        impl<'de> ::serde::de::Visitor<'de> for Visitor {
            type Value = #name #generics;

            fn expecting(&self, formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                formatter.write_str(#expecting)
            }

            fn visit_enum<V>(self, enum_visitor: V) -> Result<Self::Value, V::Error>
            where
                V: ::serde::de::EnumAccess<'de>,
            {
                use ::serde::de::VariantAccess;

                let (variant_id, variant_visitor): (u64, _) = enum_visitor.variant()?;

                #(
                    if variant_id as u32 == (#variant_id) {
                        #variant_deserialization
                    } else
                )* {
                    Err(::serde::de::Error::unknown_variant(
                        &format!("? (discriminator is {})", variant_id),
                        VARIANTS,
                    ))
                }
            }
        }
    }
}

fn generate_variant_deserializations(enum_name: &Ident, data: &DataEnum) -> Vec<QuoteOutput> {
    data.variants
        .iter()
        .map(|variant| {
            let variant_name = &variant.ident;

            match &variant.fields {
                Fields::Unit => quote! {
                    {
                        variant_visitor.unit_variant()?;
                        Ok(#enum_name::#variant_name)
                    }
                },
                Fields::Unnamed(fields) => {
                    let field_count = fields.unnamed.len();

                    if field_count == 1 {
                        quote! {
                            Ok(#enum_name::#variant_name(variant_visitor.new_type_variant()?))
                        }
                    } else {
                        unimplemented!();
                    }
                }
                Fields::Named(fields) => {
                    unimplemented!();
                }
            }
        })
        .collect()
}
