use std::collections::HashMap;
use crate::factories_booter::FactoryBootFrameworkTokenProvider;
use toml::Value;
use knockoff_logging::info;
use crate::factories_parser::factories::{Factories, Factory};
use crate::factories_parser::Provider;
use crate::framework_token_provider::FrameworkTokenProvider;
use crate::item_modifier::ItemModifierProvider;
use crate::parse_container_modifier::ParseContainerModifierProvider;
use crate::parse_provider::ParseProvider;
use crate::profile_tree_finalizer::ProfileTreeFinalizerProvider;
use crate::profile_tree_modifier::ProfileTreeModifierProvider;
use crate::token_provider::TokenProvider;
use crate::factories_parser::*;
use crate::mutable_module_modifier_provider::MutableMacroModifierProvider;


#[macro_export]
macro_rules! providers {


    ($(($ty:ident, $factory_name:ident, $factory_name_lit:literal)),*) => {

        use proc_macro2::TokenStream;
        use quote::TokenStreamExt;

        impl DelegatingProvider<Factories> for FactoriesParser {
            fn tokens(t: &Factories) -> TokenStream {
                let mut ts = TokenStream::default();
                $(
                    let tokens = $ty::tokens(t);
                    info!("Adding token for {}, {}.", $factory_name_lit, tokens.to_string().as_str());
                    ts.append_all(tokens);
                )*
                ts
            }
        }

        impl FactoriesParser {
            pub fn get_default_tokens_for(name: &str) -> TokenStream {
                $(
                    if name == $factory_name_lit {
                        return $ty::get_tokens(&vec![]);
                    }
                )*
                TokenStream::default()
            }

            pub fn get_default_tokens() -> TokenStream {
                let mut ts = TokenStream::default();
                $(
                    ts.extend($ty::get_tokens(&vec![]));
                )*
                ts
            }
        }

        $(
            impl DelegatingProvider<Factories> for $ty {
                fn tokens(t: &Factories) -> TokenStream {
                    if t.$factory_name.as_ref().is_none() {
                        $ty::get_tokens(&vec![])
                    } else {
                        let t = t.$factory_name.as_ref().unwrap();
                        let providers = &t
                            .values
                            .iter()
                            .flat_map(|val| val
                                .values()
                                .collect::<Vec<&Provider>>()
                            )
                            .collect();
                        let ts = $ty::get_tokens(providers);
                        ts
                    }
                }
            }
        )*

        impl Factories {
            pub fn get_providers(&self) -> HashMap<String, Provider> {
                let mut provider_map = HashMap::new();
                $(
                    self.insert_provider(&mut provider_map, &self.$factory_name);
                )*
                provider_map
            }
            pub fn get_factories(&mut self) -> &mut Option<Value> {
                &mut self.dependencies
            }
        }

    }
}


providers!(
    (ParseProvider, parse_provider, "parse_provider"),
    (TokenProvider, token_provider, "token_provider"),
    (FrameworkTokenProvider, framework_token_provider, "framework_token_provider"),
    (ParseContainerModifierProvider, parse_container_modifier, "parse_container_modifier"),
    (ProfileTreeModifierProvider, profile_tree_modifier_provider, "profile_tree_modifier_provider"),
    (ProfileTreeFinalizerProvider, profile_tree_finalizer, "profile_tree_finalizer"),
    (ItemModifierProvider, item_modifier, "item_modifier"),
    (MutableMacroModifierProvider, mutable_macro_modifier_provider, "mutable_macro_modifier"),
    (FactoryBootFrameworkTokenProvider, factory_framework_token_provider, "factory_framework_token_provider")
);

impl Factories {

    fn insert_provider(&self, mut provider_map: &mut HashMap<String, Provider>,
                       option: &Option<Factory>) {
        info!("Attempting to insert {:?}", option);
        option.as_ref()
            .iter()
            .flat_map(|token_provider| {
                token_provider.values.iter()
            })
            .for_each(|token_provider | {
                token_provider.iter().for_each(|t| {
                    provider_map.insert(t.0.clone(), t.1.clone());
                });
            });
    }

    fn insert_dependency(&self, name: &str, mut dep_map: &mut HashMap<String, Option<Value>>, dep: &Option<Value>
    ) {
        dep_map.insert(name.to_string(), dep.clone());
    }

}
