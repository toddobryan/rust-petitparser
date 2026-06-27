use proc_macro2::TokenStream as TokenStream2;

use quote::{format_ident, quote};
use syn::{Error, parse2};

use crate::seq_input::SeqInput;

pub fn seq_impl(input: TokenStream2) -> TokenStream2 {
    let SeqInput { parsers, target } = match parse2::<SeqInput>(input) {
        Ok(parsed) => parsed,
        Err(err) => return err.to_compile_error(),
    };

    let n: usize = parsers.len();
    if !(2..=9).contains(&n) {
        return Error::new_spanned(&parsers, "seq! macro only supports 2-9 parsers")
            .to_compile_error();
    }

    let function_name = format_ident!("seq{}", n);

    let map = match target {
        Some(target) => {
            let map_name = format_ident!("map{}", n);
            quote! { .#map_name(#target) }
        }
        None => quote! {},
    };

    quote! {
        #function_name(#parsers)#map
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
    fn fuses_arrow_target_into_map_call() {
        let input: TokenStream2 = quote! { a, b, c => |x, y, z| f(x, y, z) };
        let output = seq_impl(input).to_string();
        assert_eq!(
            output,
            quote! { seq3(a, b, c).map3(|x, y, z| f(x, y, z)) }.to_string()
        );
    }

    #[test]
    fn fuses_arrow_target_at_min_arity() {
        let input: TokenStream2 = quote! { a, b => |x, y| (y, x) };
        let output = seq_impl(input).to_string();
        assert_eq!(
            output,
            quote! { seq2(a, b).map2(|x, y| (y, x)) }.to_string()
        );
    }

    #[test]
    fn fuses_arrow_target_at_max_arity() {
        let input: TokenStream2 =
            quote! { a, b, c, d, e, f, g, h, i => |a, b, c, d, e, f, g, h, i| a };
        let output = seq_impl(input).to_string();
        assert_eq!(
            output,
            quote! { seq9(a, b, c, d, e, f, g, h, i).map9(|a, b, c, d, e, f, g, h, i| a) }
                .to_string()
        );
    }

    #[test]
    fn fuses_arrow_target_that_is_a_named_function_not_a_closure() {
        // The whole motivation for `=>`: a bare function path is just as
        // fusable as a closure literal, since the separator (not the shape
        // of what follows it) is what signals fusion.
        let input: TokenStream2 = quote! { a, b => build_message };
        let output = seq_impl(input).to_string();
        assert_eq!(
            output,
            quote! { seq2(a, b).map2(build_message) }.to_string()
        );
    }

    #[test]
    fn tolerates_an_optional_comma_before_the_arrow_too() {
        // `a, b => target` and `a, b, => target` both parse: the loop checks
        // for `=>` before requiring a comma, so the arrow alone is what ends
        // the parser list — a comma right before it is just an ordinary
        // (optional) trailing separator on that list, consumed the same way
        // it would be anywhere else in the list. The two forms aren't
        // textually identical output (the comma form leaves a harmless
        // trailing comma inside the generated `seq2(a, b,)` call), but both
        // are valid, equivalent Rust.
        let with_comma: TokenStream2 = quote! { a, b, => target };
        let without_comma: TokenStream2 = quote! { a, b => target };
        assert_eq!(
            seq_impl(with_comma).to_string(),
            quote! { seq2(a, b,).map2(target) }.to_string()
        );
        assert_eq!(
            seq_impl(without_comma).to_string(),
            quote! { seq2(a, b).map2(target) }.to_string()
        );
    }

    #[test]
    fn tolerates_a_trailing_comma_after_the_arrow_target() {
        // A closure written with a trailing comma (common for diff-
        // friendliness on multi-line bodies) shouldn't leave unconsumed
        // input behind for the top-level parse2 call to reject.
        let input: TokenStream2 = quote! { a, b => |x, y| (y, x), };
        let output = seq_impl(input).to_string();
        assert_eq!(
            output,
            quote! { seq2(a, b).map2(|x, y| (y, x)) }.to_string()
        );
    }

    #[test]
    fn does_not_fuse_a_trailing_closure_without_an_explicit_arrow() {
        // Locks in the design change: a trailing closure with no `=>` is
        // just another parser argument now, not an implicit map callback.
        let input: TokenStream2 = quote! { a, b, |x, y| x };
        let output = seq_impl(input).to_string();
        assert_eq!(output, quote! { seq3(a, b, |x, y| x) }.to_string());
    }

    #[test]
    fn handles_nested_fat_arrow_inside_a_parser_expression() {
        // A `=>` inside a `match` arm (or closure body) belongs to that
        // sub-expression, not to us — `Expr::parse` consumes it as part of
        // parsing the whole match expression before our loop ever sees it.
        let input: TokenStream2 = quote! { match v { 1 => a, _ => b }, c => target };
        let output = seq_impl(input).to_string();
        assert_eq!(
            output,
            quote! { seq2(match v { 1 => a, _ => b }, c).map2(target) }.to_string()
        );
    }

    #[test]
    fn rejects_too_few_parsers_with_arrow_target() {
        let input: TokenStream2 = quote! { a => |x| x };
        let output = seq_impl(input).to_string();
        assert!(output.contains("seq! macro only supports 2-9 parsers"));
    }
}
