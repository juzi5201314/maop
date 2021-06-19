use proc_macro::TokenStream;
use quote::{quote, ToTokens};

#[proc_macro]
pub fn stringify_lower_case(input: TokenStream) -> TokenStream {
    let output = input.to_string().to_lowercase();
    quote! (
        #output
    )
    .into()
}

#[proc_macro]
pub fn stringify_upper_case(input: TokenStream) -> TokenStream {
    let output = input.to_string().to_uppercase();
    quote! (
        #output
    )
    .into()
}
