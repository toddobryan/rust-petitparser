use proc_macro2::TokenStream as TokenStream2;

use quote::{format_ident, quote};
use syn::parse::Parser;
use syn::{Error, Expr, Token, punctuated::Punctuated};

pub fn choice_impl(input: TokenStream2) -> TokenStream2 {
    let input = match Punctuated::<Expr, Token![,]>::parse_terminated.parse2(input) {
        Ok(parsed) => parsed,
        Err(err) => return err.to_compile_error(),
    };
    let n: usize = input.len();
    if !(2..=9).contains(&n) {
        return Error::new_spanned(input, "choice! macro only supports 2-9 parsers")
            .to_compile_error();
    }

    let function_name = format_ident!("choice{}", n);

    quote! {
        #function_name(#input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_too_few_parsers() {
        let input: TokenStream2 = quote! { a };
        let output = choice_impl(input).to_string();
        assert!(output.contains("choice! macro only supports 2-9 parsers"));
    }

    #[test]
    fn rejects_too_many_parsers() {
        let input: TokenStream2 = quote! { a, b, c, d, e, f, g, h, i, j };
        let output = choice_impl(input).to_string();
        assert!(output.contains("choice! macro only supports 2-9 parsers"));
    }

    #[test]
    fn accepts_two_parsers() {
        let input: TokenStream2 = quote! { a, b };
        let output = choice_impl(input).to_string();
        assert_eq!(output, quote! { choice2(a, b) }.to_string());
    }

    #[test]
    fn accepts_nine_parsers() {
        let input: TokenStream2 = quote! { a, b, c, d, e, f, g, h, i };
        let output = choice_impl(input).to_string();
        assert_eq!(
            output,
            quote! { choice9(a, b, c, d, e, f, g, h, i) }.to_string()
        );
    }
}
