use proc_macro::TokenStream;

use crate::grammar::grammar_impl;
use crate::seq::seq_impl;

mod grammar;
mod seq;

#[proc_macro_attribute]
pub fn grammar(_attr: TokenStream, item: TokenStream) -> TokenStream {
    grammar_impl(item)
}

#[proc_macro]
pub fn seq(input: TokenStream) -> TokenStream {
    seq_impl(input)
}
