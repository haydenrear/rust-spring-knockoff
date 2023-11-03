use std::cmp::Ordering;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, TokenStreamExt, ToTokens};
use syn::{parse, parse2, parse_str, Path};
use toml::Table;
use crate::factories_parser::{Provider};

use knockoff_logging::*;
use lazy_static::lazy_static;
use std::sync::Mutex;
use syn::token::Use;
use codegen_utils::project_directory;
use crate::logger_lazy;

import_logger!("provider.rs");

pub trait DelegatingProvider<T> {
    fn tokens(t: &T) -> TokenStream;
}

pub trait ProviderProvider {
    fn create_delegating_token_provider_tokens(
        provider_type: Vec<Ident>,
        provider_idents: Vec<Ident>,
        path: &Vec<Path>
    ) -> TokenStream;

    fn create_token_provider_tokens<T: ToTokens>(provider_path_use_statement: T, provider_path: Path,
                                    provider_ident: Ident) -> TokenStream;

    fn create_token_provider(provider_item: &Provider) -> TokenStream {

        if provider_item.provider_data.is_none() {
            return TokenStream::default();
        }

        let provider_item = provider_item.provider_data.as_ref();
        if provider_item.is_none() {
            return TokenStream::default();
        }

        let provider_item = provider_item.unwrap();

        if provider_item.provider_path.is_some() || provider_item.provider_ident.is_some() {

            let provider_ident = Ident::new(&provider_item.provider_ident.as_ref().unwrap(), Span::call_site());
            let builder_path_str = provider_item.provider_path.as_ref().unwrap().as_str();


            let path = parse_str::<syn::Path>(builder_path_str).unwrap();

            if provider_item.provider_path_use_statement.as_ref().is_some()
                && provider_item.provider_path_use_statement.as_ref().unwrap().len() == 0 {
                return Self::create_token_provider_tokens(quote! {},
                                                          path, provider_ident);
            } else if provider_item.provider_path_use_statement.is_some() {
                let provider_path_use = provider_item.provider_path_use_statement
                    .as_ref().unwrap().as_str();
                let provider_path_use = parse_str::<syn::ItemUse>(provider_path_use)
                    .ok().unwrap();
                return Self::create_token_provider_tokens(
                    quote! {
                        #provider_path_use
                    }, path,
                    provider_ident
                );
            } else {
                info!("Writing provider tokens with use statement");
                return Self::create_token_provider_tokens(quote! {
                    use #path;
                }, path, provider_ident);
            }

        }

        TokenStream::default()
    }


    fn get_tokens(provider: &Vec<&Provider>) -> TokenStream {
        let mut ts = TokenStream::default();
        ts.append_all(Self::get_imports());
        provider.iter()
            .for_each(|p| ts.append_all(Self::create_token_provider(p)));
        ts.append_all(Self::get_delegating_token_provider(provider));
        ts
    }

    fn get_imports() -> TokenStream;

    fn get_delegating_token_provider(provider: &Vec<&Provider>) -> TokenStream {
        let provider_type = Self::get_provider_types(provider);

        let provider_idents = Self::get_provider_idents(provider);

        let provider_path = Self::get_provider_paths(provider);

        Self::create_delegating_token_provider_tokens(
            provider_type, provider_idents, &provider_path
        )
    }

    fn get_provider_paths(provider: &Vec<&Provider>) -> Vec<Path> {
        let provider_idents = provider.iter()
            .flat_map(|p| p.provider_data.iter())
            .flat_map(|p| p.provider_path.iter()
                .flat_map(|p| parse_str::<syn::Path>(p).ok().into_iter())
            )
            .peekable()
            .collect::<Vec<Path>>();

        provider_idents
    }

    fn get_provider_idents(provider: &Vec<&Provider>) -> Vec<Ident> {
        let provider_idents = provider.iter()
            .flat_map(|p| p.provider_data.iter())
            .flat_map(|p| p.provider_ident.iter()
                .map(|p| Ident::new(&p.to_lowercase(), Span::call_site()))
            )
            .peekable()
            .map(|p| {
                log_message!("{} is the next provider ident after.", p);
                p
            })
            .collect::<Vec<Ident>>();

        provider_idents
    }

    fn get_provider_types(provider: &Vec<&Provider>) -> Vec<Ident> {
        let provider_type = provider.iter()
            .flat_map(|p| p.provider_data.iter())
            .flat_map(|p| p.provider_ident.iter()
                .map(|p| Ident::new(p, Span::call_site()))
            )
            .collect::<Vec<Ident>>();
        provider_type
    }
}