use heck::ToUpperCamelCase;
use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, quote, quote_spanned};
use std::collections::HashSet;
use syn::parse::Parser;
use syn::punctuated::Punctuated;
use syn::visit_mut::{self, VisitMut};
use syn::{Expr, Token, parse2};

use crate::seq_input::SeqInput;

pub(crate) fn grammar_impl(item: TokenStream2) -> TokenStream2 {
    let module: syn::ItemMod = match parse2(item) {
        Ok(m) => m,
        Err(err) => return err.to_compile_error(),
    };

    let struct_name = syn::Ident::new(
        &module.ident.to_string().to_upper_camel_case(),
        module.ident.span(),
    );

    let mod_vis = &module.vis;

    let items = match &module.content {
        Some((_, items)) => items,
        None => panic!("#[grammar] requires an inline module with content (not mod foo;)"),
    };

    let functions: Vec<syn::ItemFn> = items
        .iter()
        .filter_map(|item| {
            if let syn::Item::Fn(f) = item {
                Some(f.clone())
            } else {
                None
            }
        })
        .collect();

    let mut owned_fns: Vec<syn::ItemFn> = functions.iter().map(|f| (*f).clone()).collect();

    let parser_names: HashSet<String> = owned_fns.iter().map(|f| f.sig.ident.to_string()).collect();

    let mut rewriter = ParserCallRewriter { parser_names };

    for f in &mut owned_fns {
        rewriter.visit_block_mut(&mut f.block);
    }

    let undefined_decls: Vec<TokenStream2> = owned_fns
        .iter()
        .map(|f| {
            let name = &f.sig.ident;
            let ty = parser_type(f);
            quote_spanned! { f.sig.ident.span() => let mut #name: SettableParser<#ty> = SettableParser::undefined(); }
        })
        .collect();

    let set_calls: Vec<TokenStream2> = owned_fns
        .iter()
        .map(|f| {
            let name = &f.sig.ident;
            let stmts = &f.block.stmts;
            quote_spanned! {
                f.sig.ident.span() => #name.set({ #(#stmts)* });
            }
        })
        .collect();

    let field_decls: Vec<TokenStream2> = owned_fns
        .iter()
        .map(|f| {
            let name = &f.sig.ident;
            let ty = parser_type(f);
            quote_spanned! { f.sig.ident.span() => #name: SettableParser<#ty> }
        })
        .collect();

    let field_names: Vec<_> = owned_fns.iter().map(|f| &f.sig.ident).collect();

    let start_fn = owned_fns
        .iter()
        .find(|f| f.sig.ident == "start")
        .expect("#[grammar] requires a fn start() rule");
    let start_type = parser_type(start_fn);

    let public_parsers: Vec<TokenStream2> = owned_fns
        .iter()
        .filter(|f| matches!(f.vis, syn::Visibility::Public(_)))
        .map(|f| {
            let name = &f.sig.ident;
            let ty = parser_type(f);
            let fvis = &f.vis;
            quote! {
                #fvis fn #name(&self) -> SettableParserRef<#ty> {
                    self.#name.borrow()
                }
            }
        })
        .collect();

    let output = quote! {
        #[derive(Debug)]
        #[allow(dead_code)]
        #mod_vis struct #struct_name {
            #(#field_decls),*
        }

        impl #struct_name {
            #[allow(unused_braces)]
            #mod_vis fn new() -> Self {
                #(#undefined_decls)*
                #(#set_calls)*
                Self {
                    #(#field_names,)*
                }
            }
            #(#public_parsers)*
        }

        // The grammar's immediate child is its start rule; a linter walks transitively from
        // there through every rule reachable via `SettableParserRef`.
        impl HasChildren for #struct_name {
            fn children(&self) -> Vec<std::rc::Rc<dyn HasChildren>> {
                vec![std::rc::Rc::new(self.start.clone())]
            }
        }

        impl Parser<#start_type> for #struct_name {
            fn parse_on(&self, context: &Context) -> ParseResult<#start_type> {
                self.start.parse_on(context)
            }
            fn fast_parse_on(&self, context: &Context) -> Option<usize> {
                self.start.fast_parse_on(context)
            }
        }
    };

    output
}

fn parser_type(f: &syn::ItemFn) -> &syn::Type {
    let ty = match &f.sig.output {
        syn::ReturnType::Type(_, ty) => ty,
        syn::ReturnType::Default => panic!("grammar rule must have a return type"),
    };
    // ty is &Box<Type> — now drill into impl Parser<T>

    // 1. Unwrap Box<Type> → Type::ImplTrait
    let impl_trait = match ty.as_ref() {
        syn::Type::ImplTrait(it) => it,
        _ => panic!("return type must be impl Parser<T>"),
    };

    // 2. Find the Parser<T> bound in the + separated bounds list
    for bound in &impl_trait.bounds {
        if let syn::TypeParamBound::Trait(tb) = bound {
            let seg = tb.path.segments.last().unwrap();
            if seg.ident == "Parser" {
                // 3. Extract T from the angle brackets
                if let syn::PathArguments::AngleBracketed(args) = &seg.arguments
                    && let Some(syn::GenericArgument::Type(t)) = args.args.first()
                {
                    return t;
                }
            }
        }
    }
    panic!("return type must be impl Parser<T>")
}

struct ParserCallRewriter {
    parser_names: HashSet<String>,
}

impl VisitMut for ParserCallRewriter {
    fn visit_expr_mut(&mut self, expr: &mut syn::Expr) {
        if let syn::Expr::Call(call) = &*expr
            && call.args.is_empty()
            && let syn::Expr::Path(p) = call.func.as_ref()
            && let Some(ident) = p.path.get_ident()
            && self.parser_names.contains(&ident.to_string())
        {
            let name = ident.clone();
            *expr = syn::parse_quote!(#name.borrow());
            return;
        }
        visit_mut::visit_expr_mut(self, expr);
    }

    fn visit_token_stream_mut(&mut self, tokens: &mut TokenStream2) {
        // Plain comma-separated expression list — covers choice!, seq!
        // calls with no fused map target, and any other macro whose
        // arguments are just a list of expressions.
        if let Ok(mut exprs) =
            Punctuated::<Expr, Token![,]>::parse_terminated.parse2(tokens.clone())
        {
            for expr in &mut exprs {
                self.visit_expr_mut(expr);
            }
            *tokens = exprs.to_token_stream();
            return;
        }

        // Falls back to seq!'s fused `parsers... => target` shape, which a
        // plain comma-list parse can't handle (the `=>` isn't a valid
        // continuation token). Recurses into rule calls on both sides.
        if let Ok(mut seq_input) = parse2::<SeqInput>(tokens.clone()) {
            for expr in &mut seq_input.parsers {
                self.visit_expr_mut(expr);
            }
            if let Some(target) = &mut seq_input.target {
                self.visit_expr_mut(target);
            }
            *tokens = seq_input.to_token_stream();
        }
        // Neither shape matched — leave the tokens untouched.
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Debugging aid: render generated tokens as formatted Rust source
    /// instead of `TokenStream`'s flat, space-everywhere `to_string()`.
    /// Not used by the assertions themselves (those need exact token-level
    /// spacing); call this from a test, or temporarily swap it into a
    /// panic/`eprintln!` message, when you want to actually read the output.
    fn pretty(tokens: &TokenStream2) -> String {
        let file = syn::parse2(tokens.clone()).expect("output must be a valid syn::File");
        prettyplease::unparse(&file)
    }

    /// Runs `grammar_impl` on `input` and asserts that `expected_snippet`'s
    /// tokens appear verbatim somewhere in the (much larger) generated
    /// struct/impl output. On failure, prints the *whole* output through
    /// `pretty()` so you can see what actually happened instead of staring
    /// at flat, space-everywhere token soup.
    fn assert_rewritten(input: TokenStream2, expected_snippet: TokenStream2) {
        let output_tokens = grammar_impl(input);
        let output = output_tokens.to_string();
        let expected = expected_snippet.to_string();
        assert!(
            output.contains(&expected),
            "expected output to contain `{expected_snippet}`, got:\n{}",
            pretty(&output_tokens)
        );
    }

    #[test]
    fn rewrites_rule_calls_inside_an_arbitrary_macro_invocation() {
        assert_rewritten(
            quote! {
                mod foo {
                    pub fn start() -> impl Parser<()> {
                        println!("{:?}", rest());
                        rest()
                    }
                    fn rest() -> impl Parser<()> {
                        any().map(|_| ())
                    }
                }
            },
            quote! { println!("{:?}", rest.borrow()) },
        );
    }

    #[test]
    fn rewrites_rule_calls_inside_seq_used_as_a_method_call_receiver() {
        // `seq!(...)` as the receiver of a chained `.mapN(...)` is the most
        // common shape in this codebase's actual #[grammar] modules — and,
        // unlike a bare `seq!(...)` tail expression, this puts the macro
        // call in genuine `Expr::Macro` position (nested inside an
        // `Expr::MethodCall` receiver) rather than `Stmt::Macro` position.
        assert_rewritten(
            quote! {
                mod foo {
                    pub fn start() -> impl Parser<()> {
                        seq!(a(), b()).map2(|_, _| ())
                    }
                    fn a() -> impl Parser<()> {
                        any().map(|_| ())
                    }
                    fn b() -> impl Parser<()> {
                        any().map(|_| ())
                    }
                }
            },
            quote! { seq!(a.borrow(), b.borrow()).map2(|_, _| ()) },
        );
    }

    #[test]
    fn rewrites_rule_calls_inside_seq_arrow_fusion() {
        // The `=>`-fused form (`seq!(a, b => |x, y| ...)`), with a rule call
        // buried inside the closure body — proves the rewrite recurses
        // through the target, not just the parser-list arguments. This is
        // also the path that requires the `SeqInput` fallback in
        // `visit_token_stream_mut`, since a plain comma-list parse fails on
        // the `=>` token.
        assert_rewritten(
            quote! {
                mod foo {
                    pub fn start() -> impl Parser<()> {
                        seq!(a(), b() => |x, y| combine(x, y, c()))
                    }
                    fn a() -> impl Parser<()> {
                        any().map(|_| ())
                    }
                    fn b() -> impl Parser<()> {
                        any().map(|_| ())
                    }
                    fn c() -> impl Parser<()> {
                        any().map(|_| ())
                    }
                }
            },
            quote! { seq!(a.borrow(), b.borrow() => |x, y| combine(x, y, c.borrow())) },
        );
    }

    #[test]
    fn rewrites_rule_call_used_bare_as_the_arrow_target() {
        // The motivating case for `=>`: the target need not be a closure at
        // all. A bare rule call as the target itself must still be rewritten.
        assert_rewritten(
            quote! {
                mod foo {
                    pub fn start() -> impl Parser<()> {
                        seq!(a(), b() => c())
                    }
                    fn a() -> impl Parser<()> {
                        any().map(|_| ())
                    }
                    fn b() -> impl Parser<()> {
                        any().map(|_| ())
                    }
                    fn c() -> impl Parser<()> {
                        any().map(|_| ())
                    }
                }
            },
            quote! { seq!(a.borrow(), b.borrow() => c.borrow()) },
        );
    }

    #[test]
    fn rewrites_rule_calls_inside_choice_macro() {
        assert_rewritten(
            quote! {
                mod foo {
                    pub fn start() -> impl Parser<()> {
                        choice!(a(), b())
                    }
                    fn a() -> impl Parser<()> {
                        any().map(|_| ())
                    }
                    fn b() -> impl Parser<()> {
                        any().map(|_| ())
                    }
                }
            },
            quote! { choice!(a.borrow(), b.borrow()) },
        );
    }
}
