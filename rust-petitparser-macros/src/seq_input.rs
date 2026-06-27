use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, quote};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{Expr, Token};

/// Shared parse shape for `seq!`'s fused form: `parsers... [=> target]`.
/// `=>` is the only fusion trigger — what follows it (closure, named
/// function, anything `Fn`-shaped) is passed straight to `.mapN(...)`
/// without inspecting its syntactic shape. Used by both `seq_impl` (to
/// build the `seqN(...).mapN(target)` call) and `#[grammar]`'s
/// `ParserCallRewriter` (to recurse into rule self-calls on both sides of
/// the `=>`, since a plain comma-list parse fails the moment a `seq!` call
/// uses this shape).
pub(crate) struct SeqInput {
    pub(crate) parsers: Punctuated<Expr, Token![,]>,
    pub(crate) target: Option<Expr>,
}

/// Parses `=> target`, plus an optional trailing comma after `target` (so a
/// closure written with one for diff-friendliness, e.g. `=> |x, y| { ... },`,
/// doesn't leave unconsumed input behind for the top-level `parse2` call to
/// reject).
fn parse_arrow_target(input: ParseStream) -> syn::Result<Expr> {
    input.parse::<Token![=>]>()?;
    let target = input.parse()?;
    if input.peek(Token![,]) {
        input.parse::<Token![,]>()?;
    }
    Ok(target)
}

impl Parse for SeqInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut parsers = Punctuated::new();
        loop {
            if input.is_empty() {
                break;
            }
            if input.peek(Token![=>]) {
                let target = parse_arrow_target(input)?;
                return Ok(SeqInput {
                    parsers,
                    target: Some(target),
                });
            }
            parsers.push_value(input.parse()?);
            if input.is_empty() {
                break;
            }
            if input.peek(Token![=>]) {
                let target = parse_arrow_target(input)?;
                return Ok(SeqInput {
                    parsers,
                    target: Some(target),
                });
            }
            parsers.push_punct(input.parse::<Token![,]>()?);
        }
        Ok(SeqInput {
            parsers,
            target: None,
        })
    }
}

impl ToTokens for SeqInput {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        self.parsers.to_tokens(tokens);
        if let Some(target) = &self.target {
            tokens.extend(quote! { => #target });
        }
    }
}
