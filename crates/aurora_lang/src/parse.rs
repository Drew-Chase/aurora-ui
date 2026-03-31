use syn::LitStr;
use syn::parse::{Parse, ParseStream};

/// Input to the `lang!` macro: a single string literal path.
pub struct LangInput {
    pub path: LitStr,
}

impl Parse for LangInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let path: LitStr = input.parse()?;
        Ok(Self { path })
    }
}
