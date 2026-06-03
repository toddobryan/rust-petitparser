use heck::ToUpperCamelCase;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use std::collections::HashSet;
use syn::parse_macro_input;
use syn::visit_mut::{self, VisitMut};

#[proc_macro_attribute]
pub fn grammar(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let module = parse_macro_input!(item as syn::ItemMod);

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
            quote! { let mut #name: SettableParser<#ty> = SettableParser::undefined(); }
        })
        .collect();

    let set_calls: Vec<TokenStream2> = owned_fns
        .iter()
        .map(|f| {
            let name = &f.sig.ident;
            let stmts = &f.block.stmts;
            quote! {
                #name.set({ #(#stmts)* });
            }
        })
        .collect();

    let field_decls: Vec<TokenStream2> = owned_fns
        .iter()
        .map(|f| {
            let name = &f.sig.ident;
            let ty = parser_type(f);
            quote! { #name: SettableParser<#ty> }
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
        #mod_vis struct #struct_name {
            #(#field_decls),*
        }

        impl #struct_name {
            #mod_vis fn new() -> Self {
                #(#undefined_decls)*
                #(#set_calls)*
                Self {
                    #(#field_names,)*
                }
            }
            #(#public_parsers)*
        }

        impl Parser<#start_type> for #struct_name {
            fn parse_on(&self, context: &Context) -> ParseResult<#start_type> {
                self.start.parse_on(context)
            }
            fn fast_parse_on(&self, buffer: Rc<[char]>, position: usize) -> Option<usize> {
                self.start.fast_parse_on(buffer, position)
            }
        }
    };

    output.into()
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
}
