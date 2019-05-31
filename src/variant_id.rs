use crate::proc_macro::TokenStream;
use proc_macro2::TokenStream as QuoteOutput;
use quote::{ToTokens, TokenStreamExt};

#[derive(Clone, Copy, Debug, Default)]
pub struct VariantId {
    offset: u32,
}

impl VariantId {
    pub fn from_value(value: u32) -> Self {
        VariantId { offset: value }
    }

    pub fn next(&self) -> Self {
        VariantId {
            offset: self.offset + 1,
        }
    }
}

impl ToTokens for VariantId {
    fn to_tokens(&self, token_stream: &mut QuoteOutput) {
        token_stream.append_all(&[self.offset]);
    }
}
