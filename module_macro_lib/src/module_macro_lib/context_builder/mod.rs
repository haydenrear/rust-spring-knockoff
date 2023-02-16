use std::collections::HashMap;
use proc_macro2::{Ident, TokenStream};
use quote::{quote, TokenStreamExt, ToTokens};
use syn::Type;
use knockoff_logging::{initialize_log, use_logging};
use crate::module_macro_lib::module_tree::{Bean, BeanDefinitionType, InjectableTypeKey, Profile};
use crate::module_macro_lib::parse_container::ParseContainer;
use crate::module_macro_lib::knockoff_context_builder::ApplicationContextGenerator;

pub struct ContextBuilder;

use crate::module_macro_lib::logging::executor;
use crate::module_macro_lib::logging::StandardLoggingFacade;

use_logging!();
initialize_log!();

impl ContextBuilder {

    pub fn build_token_stream(parse_container: &mut ParseContainer) -> TokenStream {

        parse_container.log_app_container_info();
        parse_container.build_injectable();

        let mut token = quote! {};

        let mut profile_map = HashMap::new();

        parse_container.injectable_types_map.injectable_types.iter()
            .flat_map(|bean_def_type_profile| bean_def_type_profile.1.iter()
                .map(move |bean_def_type| (bean_def_type_profile.0, bean_def_type))
            )
            .for_each(|bean_def_type_profile| {
                match bean_def_type_profile.1 {
                    BeanDefinitionType::Abstract { bean, dep_type } => {
                        Self::implement_abstract_autowire(&mut token, bean, bean_def_type_profile.0);
                        Self::insert_into_profile_map(&mut profile_map, bean_def_type_profile, bean);
                    }
                    BeanDefinitionType::Concrete { bean } => {
                        Self::implement_concrete_autowire(&mut token, bean, bean_def_type_profile.0);
                        Self::insert_into_profile_map(&mut profile_map, bean_def_type_profile, bean);
                    }
                }
            });

        Self::finish_writing_factory(&mut token, profile_map);

        token
    }

    fn insert_into_profile_map(mut profile_map: &mut HashMap<Profile, Vec<Bean>>, bean_def_type_profile: (&Profile, &BeanDefinitionType), bean: &Bean) {
        if profile_map.contains_key(bean_def_type_profile.0) {
            profile_map.get_mut(bean_def_type_profile.0)
                .map(|beans| beans.push(bean.clone()));
        } else {
            let bean_vec = vec![bean.clone()];
            profile_map.insert(bean_def_type_profile.0.clone(), bean_vec);
        }
    }

    fn finish_writing_factory(token: &mut TokenStream, beans: HashMap<Profile, Vec<Bean>>) {

        beans.iter().for_each(|profile_type| {
            log_message!("Creating bean factory for profile type: {}.", profile_type.0.profile.clone());
            let listable_bean_factory = ApplicationContextGenerator::new_listable_bean_factory(
                profile_type.1.clone(),
                profile_type.0.clone()
            );

            token.extend(listable_bean_factory.into_iter());

            token.append_all(ApplicationContextGenerator::finish_abstract_factory(vec![profile_type.0.profile.clone()]));
        })

    }

    fn implement_abstract_autowire(mut token: &mut TokenStream, token_type: &Bean, profile: &Profile) {

        let (field_types, identifiers) = Self::get_field_ids(token_type);

        if token_type.struct_type.is_some() {
            let struct_type = token_type.struct_type.clone()
                .unwrap();
            Self::implement_abstract_code(&mut token, &field_types, &identifiers, &struct_type);
        } else if token_type.ident.is_some() {
            let struct_type = token_type.ident.clone()
                .unwrap();
            Self::implement_abstract_code(&mut token, &field_types, &identifiers, &struct_type);
        }
    }

    fn implement_concrete_autowire(mut token: &mut TokenStream, token_type: &Bean, profile: &Profile) {

        let (field_types, identifiers) = Self::get_field_ids(token_type);

        if token_type.struct_type.is_some() {
            let struct_type = token_type.struct_type.clone()
                .unwrap();
            Self::implement_autowire_code(&mut token, &field_types, &identifiers, &struct_type);
        } else if token_type.ident.is_some() {
            let struct_type = token_type.ident.clone()
                .unwrap();
            Self::implement_autowire_code(&mut token, &field_types, &identifiers, &struct_type);
        }
    }

    fn get_field_ids(token_type: &Bean) -> (Vec<Type>, Vec<Ident>) {
        let field_types = token_type.deps_map
            .clone().iter()
            .map(|d| d.bean_info.type_of_field.clone())
            .collect::<Vec<Type>>();

        let identifiers = token_type.deps_map
            .clone().iter()
            .flat_map(|d| {
                match &d.bean_info.field.ident {
                    None => {
                        vec![]
                    }
                    Some(identifier) => {
                        vec![identifier.clone()]
                    }
                }
            })
            .collect::<Vec<Ident>>();
        (field_types, identifiers)
    }

    fn implement_autowire_code<T: ToTokens>(token: &mut TokenStream, field_types: &Vec<Type>, identifiers: &Vec<Ident>, struct_type: &T) {
        log_message!("Implementing container for {}.", struct_type.to_token_stream().to_string().clone());

        let this_struct_impl = ApplicationContextGenerator::gen_autowire_code_gen_concrete(
            &field_types, &identifiers, &struct_type
        );

        token.append_all(this_struct_impl);
    }

    fn implement_abstract_code<T: ToTokens>(token: &mut TokenStream, field_types: &Vec<Type>, identifiers: &Vec<Ident>, struct_type: &T) {
        log_message!("Implementing container for {}.", struct_type.to_token_stream().to_string().clone());

        let this_struct_impl = ApplicationContextGenerator::gen_autowire_code_gen_abstract(
            &field_types, &identifiers, &struct_type
        );

        token.append_all(this_struct_impl);
    }

    pub fn build_concrete_types(map: &HashMap<String, Bean>) -> HashMap<InjectableTypeKey, Bean> {
        let mut return_map = HashMap::new();
        for i_type in map.iter() {
            return_map.insert(InjectableTypeKey {
                underlying_type: i_type.0.clone(),
                impl_type: None,
                profile: i_type.1.profile.clone()
            }, i_type.1.clone());
        }
        return_map
    }


}