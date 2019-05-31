use proc_macro2::{Punct, Spacing, TokenStream as QuoteOutput, TokenTree};
use quote::{quote, ToTokens, TokenStreamExt};

#[derive(Clone, Debug, Default)]
pub struct VariantId {
    expression: Option<QuoteOutput>,
    offset: u32,
}

impl VariantId {
    pub fn from_expression(expression: QuoteOutput) -> Self {
        VariantId {
            expression: Some(expression),
            offset: 0,
        }
    }

    pub fn from_value(value: u32) -> Self {
        VariantId {
            expression: None,
            offset: value,
        }
    }

    pub fn next(&self) -> Self {
        VariantId {
            expression: self.expression.clone(),
            offset: self.offset + 1,
        }
    }
}

impl ToTokens for VariantId {
    fn to_tokens(&self, token_stream: &mut QuoteOutput) {
        if let Some(expression) = self.expression.clone() {
            token_stream.append_all(expression);
            token_stream.append(TokenTree::Punct(Punct::new('+', Spacing::Alone)));
        }

        token_stream.append_all(&[self.offset]);
    }

    fn into_token_stream(self) -> QuoteOutput {
        match self.expression {
            Some(mut expression) => {
                expression.append(TokenTree::Punct(Punct::new('+', Spacing::Alone)));
                expression.append_all(&[self.offset]);
                expression
            }
            None => {
                let id = self.offset;
                quote! { #id }
            }
        }
    }
}
