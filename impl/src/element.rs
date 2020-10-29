use crate::attr;
use proc_macro2::{Span, TokenStream, TokenTree};
use quote::{format_ident, quote, quote_spanned};
use std::iter::FromIterator;
use syn::parse::{Error, Parse, ParseStream, Result};
use syn::punctuated::Punctuated;
use syn::{
    braced, parenthesized, parse_quote, Abi, Attribute, BareFnArg, BoundLifetimes, GenericParam,
    Generics, Ident, Path, ReturnType, Token, Type, TypeBareFn, Visibility, WhereClause,
};

pub struct Element {
    attrs: Vec<Attribute>,
    vis: Visibility,
    ident: Ident,
    ty: Type,
    expr: TokenStream,
    orig_item: Option<TokenStream>,
    start_span: Span,
    end_span: Span,
}

impl Parse for Element {
    fn parse(input: ParseStream) -> Result<Self> {
        let start = input.cursor();
        let attrs = input.call(Attribute::parse_outer)?;
        let item = input.cursor();
        let vis: Visibility = input.parse()?;
        let static_token: Option<Token![static]> = input.parse()?;
        if static_token.is_some() {
            let ident: Ident = input.parse()?;
            input.parse::<Token![:]>()?;
            let start_span = input.span();
            let ty: Type = input.parse()?;
            let end_span = quote!(#ty).into_iter().last().unwrap().span();
            input.parse::<Token![=]>()?;
            let mut expr_semi = Vec::from_iter(input.parse::<TokenStream>()?);
            if let Some(tail) = expr_semi.pop() {
                syn::parse2::<Token![;]>(TokenStream::from(tail))?;
            }
            let expr = TokenStream::from_iter(expr_semi);
            Ok(Element {
                attrs,
                vis,
                ident,
                ty,
                expr,
                orig_item: None,
                start_span,
                end_span,
            })
        } else {
            let constness: Option<Token![const]> = input.parse()?;
            let asyncness: Option<Token![async]> = input.parse()?;
            let unsafety: Option<Token![unsafe]> = input.parse()?;
            let abi: Option<Abi> = input.parse()?;
            let fn_token: Token![fn] = input.parse().map_err(|_| {
                Error::new_spanned(
                    item.token_stream(),
                    "distributed element must be either static or function item",
                )
            })?;
            let ident: Ident = input.parse()?;
            let generics: Generics = input.parse()?;

            let content;
            let paren_token = parenthesized!(content in input);
            let mut inputs = Punctuated::new();
            while !content.is_empty() {
                content.parse::<Option<Token![mut]>>()?;
                let ident = if let Some(wild) = content.parse::<Option<Token![_]>>()? {
                    Ident::from(wild)
                } else {
                    content.parse()?
                };
                let colon_token: Token![:] = content.parse()?;
                let ty: Type = content.parse()?;
                inputs.push_value(BareFnArg {
                    attrs: Vec::new(),
                    name: Some((ident, colon_token)),
                    ty,
                });
                if !content.is_empty() {
                    let comma: Token![,] = content.parse()?;
                    inputs.push_punct(comma);
                }
            }

            let output: ReturnType = input.parse()?;
            let where_clause: Option<WhereClause> = input.parse()?;

            let content;
            braced!(content in input);
            content.parse::<TokenStream>()?;

            if let Some(constness) = constness {
                return Err(Error::new_spanned(
                    constness,
                    "const fn distributed slice element is not supported",
                ));
            }

            if let Some(asyncness) = asyncness {
                return Err(Error::new_spanned(
                    asyncness,
                    "async fn distributed slice element is not supported",
                ));
            }

            let lifetimes = if generics.params.is_empty() {
                None
            } else {
                let mut bound = BoundLifetimes {
                    for_token: Token![for](generics.lt_token.unwrap().span),
                    lt_token: generics.lt_token.unwrap(),
                    lifetimes: Punctuated::new(),
                    gt_token: generics.gt_token.unwrap(),
                };
                for param in generics.params.into_pairs() {
                    let (param, punct) = param.into_tuple();
                    match param {
                        GenericParam::Lifetime(lifetime) => {
                            bound.lifetimes.push_value(lifetime);
                            if let Some(punct) = punct {
                                bound.lifetimes.push_punct(punct);
                            }
                        }
                        _ => {
                            return Err(Error::new_spanned(
                                param,
                                "cannot have generic parameters on distributed slice element",
                            ))
                        }
                    }
                }
                Some(bound)
            };

            if let Some(where_clause) = where_clause {
                return Err(Error::new_spanned(
                    where_clause,
                    "where-clause is not allowed on distributed slice elements",
                ));
            }

            let start_span = item.span();
            let end_span = quote!(#output)
                .into_iter()
                .last()
                .as_ref()
                .map_or(paren_token.span, TokenTree::span);
            let attrs = vec![parse_quote! {
                #[allow(non_upper_case_globals)]
            }];
            let vis = Visibility::Inherited;
            let expr = parse_quote!(#ident);
            let ty = Type::BareFn(TypeBareFn {
                lifetimes,
                unsafety,
                abi,
                fn_token,
                paren_token,
                inputs,
                variadic: None,
                output,
            });
            let ident = format_ident!("_LINKME_ELEMENT_{}", ident);
            let orig_item = Some(start.token_stream());

            Ok(Element {
                attrs,
                vis,
                ident,
                ty,
                expr,
                orig_item,
                start_span,
                end_span,
            })
        }
    }
}

pub fn expand(path: Path, pos: usize, input: Element) -> TokenStream {
    let mut attrs = input.attrs;
    let vis = input.vis;
    let ident = input.ident;
    let ty = input.ty;
    let expr = input.expr;
    let orig_item = input.orig_item;

    let linkme_path = match attr::linkme_path(&mut attrs) {
        Ok(path) => path,
        Err(err) => return err.to_compile_error(),
    };

    let new = quote_spanned!(input.start_span=> __new);
    let uninit = quote_spanned!(input.end_span=> #new());

    TokenStream::from(quote! {
        #path ! {
            #(#attrs)*
            #vis static #ident : #ty = {
                unsafe fn __typecheck(_: #linkme_path::private::Void) {
                    let #new = #linkme_path::private::value::<#ty>;
                    #linkme_path::DistributedSlice::private_typecheck(#path, #uninit)
                }

                #expr
            };
        }

        #orig_item
    })
}
