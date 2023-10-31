use quote::ToTokens;
use syn::Attribute;
use codegen_utils::syn_helper::SynHelper;

use module_macro_shared::profile_tree::ProfileBuilder;
use knockoff_logging::*;
use lazy_static::lazy_static;
use std::sync::Mutex;
use codegen_utils::project_directory;
use crate::logger_lazy;
import_logger!("util.rs");


pub struct ParseUtil;

impl ParseUtil {

    pub fn get_qualifiers(attr: &Vec<Attribute>) -> Vec<String> {
        Self::get_attr_csv(&attr, &Self::get_qualifier_attr_names())
    }

    pub fn get_qualifier_attr_names() -> Vec<&'static str> {
        vec!["prototype", "service", "bean", "controller", "rest_controller", "component", "qualifier"]
    }

    pub fn get_prototype_names(attr: &Vec<Attribute>) -> Option<Vec<String>> {
        Self::get_attr_csv_if_exists(&attr, &vec!["prototype"])
    }

    pub fn get_attr_csv_if_exists(attr: &Vec<Attribute>, x: &Vec<&str>) -> Option<Vec<String>> {
        if Self::does_attr_exist(&attr, x) {
            return Some(Self::get_attr_csv(&attr, x))
        }
        None
    }

    pub fn get_singleton_names(attr: &Vec<Attribute>) -> Option<Vec<String>> {
        Self::get_attr_csv_if_exists(&attr, &vec!["service", "enum_service"])
    }

    pub fn get_profile(attr: &Vec<Attribute>) -> Vec<ProfileBuilder> {
        Self::get_attr_csv(&attr, &vec!["profile"]).iter().map(|profile| ProfileBuilder {profile: profile.clone()})
            .collect::<Vec<ProfileBuilder>>()
    }

    fn get_attr_csv(attr: &Vec<Attribute>, x: &Vec<&str>) -> Vec<String> {
        let found = Self::get_attr_path(&attr, x)
            .map(|profile| profile.split(", ")
                .map(|profile| profile.to_string())
                .map(|mut profile| profile.replace(" ", ""))
                .collect::<Vec<String>>()
            )
            .or(Some(vec![]))
            .unwrap();
        found
    }

    fn get_attr_path(attrs: &Vec<Attribute>, to_parse: &Vec<&str>) -> Option<String> {
        SynHelper::get_attr_from_vec(attrs, to_parse)
    }

    pub fn does_attr_exist(attrs: &Vec<Attribute>, to_parse: &Vec<&str>) -> bool {
        SynHelper::get_attr_from_vec(&attrs, to_parse)
            .map(|_| true)
            .or(Some(false))
            .unwrap()
    }

}