use std::cmp::Ordering;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, TokenStreamExt, ToTokens};
use syn::{parse_str, Path};
use toml::Table;
use crate::factories_parser::{Provider};

pub trait DelegatingProvider {
    fn tokens() -> TokenStream;
}

pub trait ProviderProvider {

    fn create_delegating_token_provider_tokens(provider_type: Vec<Ident>, provider_idents: Vec<Ident>) -> TokenStream;
    fn create_token_provider_tokens(provider_path: Path, provider_ident: Ident) -> TokenStream;

    fn create_token_provider(provider_item: &Provider) -> TokenStream {

        if provider_item.provider_data.is_none() {
            return TokenStream::default();
        }

        let provider_item = provider_item.provider_data.as_ref().unwrap();

        if provider_item.provider_path.is_some() || provider_item.provider_ident.is_some() {

            let provider_ident = Ident::new(&provider_item.provider_ident.as_ref().unwrap(), Span::call_site());
            let builder_path_str = provider_item.provider_path.as_ref().unwrap().as_str();

            let path = parse_str::<syn::Path>(builder_path_str).unwrap();

            return Self::create_token_provider_tokens(path, provider_ident);
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

        Self::create_delegating_token_provider_tokens(provider_type, provider_idents)
    }

    fn get_provider_idents(provider: &Vec<&Provider>) -> Vec<Ident> {
        let provider_idents = provider.iter()
            .flat_map(|p| p.provider_data.as_ref()
                .map(|p| vec![p]).or(Some(vec![])).unwrap()
            )
            .flat_map(|p| p.provider_ident.as_ref()
                .map(|p| vec![Ident::new(&p.to_lowercase(), Span::call_site())])
                .or(Some(vec![]))
                .unwrap()
            )
            .collect::<Vec<Ident>>();
        provider_idents
    }

    fn get_provider_types(provider: &Vec<&Provider>) -> Vec<Ident> {
        let provider_type = provider.iter()
            .flat_map(|p| p.provider_data.as_ref()
                .map(|p| vec![p]).or(Some(vec![])).unwrap()
            )
            .flat_map(|p| p.provider_ident.as_ref()
                .map(|p| vec![Ident::new(p, Span::call_site())])
                .or(Some(vec![]))
                .unwrap()
            )
            .collect::<Vec<Ident>>();
        provider_type
    }
}