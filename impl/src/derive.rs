use crate::linker;
use proc_macro2::{TokenStream, TokenTree};
use quote::quote;
use syn::parse::{Parse, ParseStream, Result};
use syn::{braced, parenthesized, token, Ident, LitStr};

pub struct Enum {
    linkme_ident: Ident,
    linkme_macro: Ident,
}

impl Parse for Enum {
    fn parse(input: ParseStream) -> Result<Self> {
        // #[doc(hidden)]
        // enum #link_section_enum_dummy {
        //     _Ident = (#linkme_ident, 0).1,
        //     _Macro = (#linkme_macro, 1).1,
        // }

        while !input.peek(token::Brace) {
            input.parse::<TokenTree>()?;
        }
        let variants;
        braced!(variants in input);

        while !variants.peek(token::Paren) {
            variants.parse::<TokenTree>()?;
        }
        let discriminant;
        parenthesized!(discriminant in variants);
        let linkme_ident = discriminant.parse::<LitStr>()?.parse::<Ident>()?;
        discriminant.parse::<TokenStream>()?;

        while !variants.peek(token::Paren) {
            variants.parse::<TokenTree>()?;
        }
        let discriminant;
        parenthesized!(discriminant in variants);
        let linkme_macro = discriminant.parse::<LitStr>()?.parse::<Ident>()?;
        discriminant.parse::<TokenStream>()?;
        variants.parse::<TokenStream>()?;

        Ok(Enum {
            linkme_ident,
            linkme_macro,
        })
    }
}

pub fn expand(input: Enum) -> TokenStream {
    let ident = input.linkme_ident;
    let ident_macro = input.linkme_macro;
    let linux_section = linker::linux::section(&ident);
    let macos_section = linker::macos::section(&ident);
    let none_section = linker::none::section(&ident);
    let windows_section = linker::windows::section(&ident);

    TokenStream::from(quote! {
        #[doc(hidden)]
        #[macro_export]
        macro_rules! #ident_macro {
            (
                #![linkme_macro = $macro:path]
                #![linkme_sort_key = $key:tt]
                $item:item
            ) => {
                $macro ! {
                    #![linkme_linux_section = concat!(#linux_section, $key)]
                    #![linkme_macos_section = concat!(#macos_section, $key)]
                    #![linkme_windows_section = concat!(#windows_section, $key)]
                    #![linkme_none_section = concat!(#none_section, $key)]
                    $item
                }
            };
            (
                #![linkme_linux_section = $linux_section:expr]
                #![linkme_macos_section = $macos_section:expr]
                #![linkme_windows_section = $windows_section:expr]
                #![linkme_none_section = $none_section:expr]
                $item:item
            ) => {
                #[used]
                #[cfg_attr(target_os = "linux", link_section = $linux_section)]
                #[cfg_attr(target_os = "macos", link_section = $macos_section)]
                #[cfg_attr(target_os = "windows", link_section = $windows_section)]
                #[cfg_attr(target_os = "none", link_section = $none_section)]
                $item
            };
            ($item:item) => {
                #[used]
                #[cfg_attr(target_os = "linux", link_section = #linux_section)]
                #[cfg_attr(target_os = "macos", link_section = #macos_section)]
                #[cfg_attr(target_os = "windows", link_section = #windows_section)]
                #[cfg_attr(target_os = "none", link_section = #none_section)]
                $item
            };
        }
    })
}
