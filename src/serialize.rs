use crate::{collect_variant_ids, get_enum_data, proc_macro::TokenStream};
use proc_macro2::{Span, TokenStream as QuoteOutput};
use quote::quote;
use syn::{DataEnum, DeriveInput, Field, Fields, Ident, LitStr};

pub fn derive(input: DeriveInput) -> TokenStream {
    let name = &input.ident;
    let data = get_enum_data(&input);

    let variant = collect_variants(name, data);
    let serialization = generate_serializations(name, data);

    let output = quote! {
        impl ::serde::Serialize for #name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: ::serde::Serializer,
            {
                match self {
                    #( #variant => #serialization ),*
                }
            }
        }
    };

    output.into()
}

fn collect_variants(enum_name: &Ident, data: &DataEnum) -> Vec<QuoteOutput> {
    data.variants
        .iter()
        .map(|variant| {
            let name = &variant.ident;
            let fields = match &variant.fields {
                Fields::Unit => quote!(),
                Fields::Unnamed(fields) => {
                    let field_names = generate_field_names(fields.unnamed.iter());

                    quote! {
                        ( #( #field_names ),* )
                    }
                }
                Fields::Named(fields) => {
                    let field_names = generate_field_names(fields.named.iter());

                    quote! {
                        { #( #field_names ),* }
                    }
                }
            };

            quote! { #enum_name::#name #fields }
        })
        .collect()
}

fn generate_field_names<'a>(fields: impl Iterator<Item = &'a Field>) -> Vec<QuoteOutput> {
    fields
        .zip(0..)
        .map(|(field, index)| match &field.ident {
            Some(identifier) => quote! { #identifier },
            None => {
                let id = LitStr::new(&format!("_{}", index), Span::call_site());

                quote! { #id }
            }
        })
        .collect()
}

fn generate_serializations(name: &Ident, data: &DataEnum) -> Vec<QuoteOutput> {
    let name_str = LitStr::new(&name.to_string(), Span::call_site());
    let ids = collect_variant_ids(data);

    data.variants
        .iter()
        .zip(ids)
        .map(|(variant, id)| {
            let variant_name = LitStr::new(&variant.ident.to_string(), Span::call_site());

            match &variant.fields {
                Fields::Unit => quote! {
                    serializer.serialize_unit_variant(#name_str, #id as u32, #variant_name)
                },
                Fields::Unnamed(fields) => {
                    let field_count = fields.unnamed.len();
                    let field_names = generate_field_names(fields.unnamed.iter());

                    if field_count == 1 {
                        let value = &field_names[0];

                        quote! {
                            seralizer.serialize_newtype_variant(
                                #name_str,
                                #id as u32,
                                #variant_name,
                                #value,
                            )
                        }
                    } else {
                        quote! {
                            let tuple_serializer = serializer.serialize_tuple_variant(
                                #name_str,
                                #id as u32,
                                #variant_name,
                                #field_count,
                            )?;

                            #( tuple_serializer.serialize_field( #field_names )?; )*
                            tuple_serializer.end();
                        }
                    }
                }
                Fields::Named(fields) => {
                    let field_count = fields.named.len();
                    let field_names = generate_field_names(fields.named.iter());

                    quote! {
                        let struct_serializer = serializer.serialize_struct_variant(
                            #name_str,
                            #id as u32,
                            #variant_name,
                            #field_count,
                        )?;

                        #( struct_serializer.serialize_field( #field_names )?; )*
                        struct_serializer.end();
                    }
                }
            }
        })
        .collect()
}
