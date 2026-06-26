use proc_macro::TokenStream;

use crate::choice::choice_impl;
use crate::grammar::grammar_impl;
use crate::seq::seq_impl;

mod choice;
mod grammar;
mod seq;

#[proc_macro_attribute]
pub fn grammar(_attr: TokenStream, item: TokenStream) -> TokenStream {
    grammar_impl(item)
}

#[proc_macro]
pub fn seq(input: TokenStream) -> TokenStream {
    seq_impl(input.into()).into()
}

#[proc_macro]
pub fn choice(input: TokenStream) -> TokenStream {
    choice_impl(input.into()).into()
}
