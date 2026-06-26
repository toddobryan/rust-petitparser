use proc_macro2::TokenStream as TokenStream2;

use quote::{format_ident, quote};
use syn::ExprClosure;
use syn::parse::Parser;
use syn::{Error, Expr, Token, punctuated::Punctuated};

pub fn seq_impl(input: TokenStream2) -> TokenStream2 {
    let input = match Punctuated::<Expr, Token![,]>::parse_terminated.parse2(input) {
        Ok(parsed) => parsed,
        Err(err) => return err.to_compile_error(),
    };

    let mut exprs: Vec<Expr> = input.clone().into_iter().collect();

    let opt_closure: Option<ExprClosure> = match exprs.pop() {
        Some(Expr::Closure(c)) => Some(c),
        Some(other) => {
            exprs.push(other);
            None
        }
        None => None,
    };

    let n: usize = exprs.len();
    if !(2..=9).contains(&n) {
        return Error::new_spanned(input, "seq! macro only supports 2-9 parsers")
            .to_compile_error();
    }

    let function_name = format_ident!("seq{}", n);

    let map = match opt_closure {
        Some(closure) => {
            let map_name = format_ident!("map{}", n);
            quote! { .#map_name(#closure) }
        }
        None => quote! {},
    };

    quote! {
        #function_name(#(#exprs),*)#map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_too_few_parsers() {
        let input: TokenStream2 = quote! { a };
        let output = seq_impl(input).to_string();
        assert!(output.contains("seq! macro only supports 2-9 parsers"));
    }

    #[test]
    fn rejects_too_many_parsers() {
        let input: TokenStream2 = quote! { a, b, c, d, e, f, g, h, i, j };
        let output = seq_impl(input).to_string();
        assert!(output.contains("seq! macro only supports 2-9 parsers"));
    }

    #[test]
    fn accepts_two_parsers() {
        let input: TokenStream2 = quote! { a, b };
        let output = seq_impl(input).to_string();
        assert_eq!(output, quote! { seq2(a, b) }.to_string());
    }

    #[test]
    fn accepts_nine_parsers() {
        let input: TokenStream2 = quote! { a, b, c, d, e, f, g, h, i };
        let output = seq_impl(input).to_string();
        assert_eq!(
            output,
            quote! { seq9(a, b, c, d, e, f, g, h, i) }.to_string()
        );
    }

    #[test]
    fn fuses_trailing_closure_into_map_call() {
        let input: TokenStream2 = quote! { a, b, c, |x, y, z| f(x, y, z) };
        let output = seq_impl(input).to_string();
        assert_eq!(
            output,
            quote! { seq3(a, b, c).map3(|x, y, z| f(x, y, z)) }.to_string()
        );
    }

    #[test]
    fn fuses_trailing_closure_at_min_arity() {
        let input: TokenStream2 = quote! { a, b, |x, y| (y, x) };
        let output = seq_impl(input).to_string();
        assert_eq!(
            output,
            quote! { seq2(a, b).map2(|x, y| (y, x)) }.to_string()
        );
    }

    #[test]
    fn fuses_trailing_closure_at_max_arity() {
        let input: TokenStream2 =
            quote! { a, b, c, d, e, f, g, h, i, |a, b, c, d, e, f, g, h, i| a };
        let output = seq_impl(input).to_string();
        assert_eq!(
            output,
            quote! { seq9(a, b, c, d, e, f, g, h, i).map9(|a, b, c, d, e, f, g, h, i| a) }
                .to_string()
        );
    }

    #[test]
    fn rejects_too_few_parsers_with_trailing_closure() {
        let input: TokenStream2 = quote! { a, |x| x };
        let output = seq_impl(input).to_string();
        assert!(output.contains("seq! macro only supports 2-9 parsers"));
    }
}
