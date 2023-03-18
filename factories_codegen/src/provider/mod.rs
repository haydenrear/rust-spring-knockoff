use std::cmp::Ordering;
use proc_macro2::{Ident, TokenStream};
use quote::ToTokens;
use crate::factories_parser::Dependency;

#[derive(Clone)]
pub struct Provider {
    pub providers: Vec<ProviderItem>,
}

pub trait DelegatingProvider {
    fn tokens() -> TokenStream;
    fn deps() -> Vec<ProviderItem>;
}

#[derive(Clone)]
pub struct ProviderItem {
    pub name: String,
    pub provider_path: syn::Path,
    pub provider_ident: Ident,
    pub path: Option<String>,
    pub dep_name: String,
    pub version: Option<String>,
    pub deps: Vec<Dependency>
}

impl Eq for ProviderItem {}

impl PartialEq<Self> for ProviderItem {
    fn eq(&self, other: &Self) -> bool {
        self.name.eq(&other.name)
    }
}

impl PartialOrd<Self> for ProviderItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.name.partial_cmp(&other.name)
    }
}

impl Ord for ProviderItem {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name.cmp(&other.name)
    }
}