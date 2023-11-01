use syn::{Attribute, FnArg, GenericParam, Generics, ItemFn, Pat, PatType, ReturnType, Type, TypeParam, WherePredicate};
use std::any::{Any, TypeId};
use std::collections::{HashMap, HashSet};
use std::ops::Deref;
use proc_macro2::{Ident, TokenStream};
use quote::ToTokens;
use codegen_utils::syn_helper::SynHelper;
use crate::module_macro_lib::item_parser::{create_new_gens, GenericTy, get_all_generic_ty_bounds, get_profiles, ItemParser};
use module_macro_shared::parse_container::ParseContainer;

use knockoff_logging::*;
use lazy_static::lazy_static;
use std::sync::Mutex;
use syn::ext::IdentExt;
use codegen_utils::project_directory;
use module_macro_shared::bean::{BeanDefinition, BeanPath, BeanPathParts, BeanType};
use crate::logger_lazy;
import_logger!("item_fn_parser.rs");

use module_macro_shared::bean::BeanPathParts::FnType;
use module_macro_shared::functions::{FunctionType, ModulesFunctions};
use crate::module_macro_lib::bean_parser::bean_dependency_path_parser::BeanDependencyPathParser;
use crate::module_macro_lib::util::ParseUtil;

pub struct ItemFnParser;


/// FactoryFn parsed as bean
///  -> FactoryFn bean adds all abstract types qualifier injects as traits_impl
///  -> BeanDefinitionInfo includes fields that were injected and names of fields as per normal,
///     but through the FnArg
///  -> BeanFactory created for concrete factory_fn and all abstract types as per normal
///  -> BeanFactory used to get factory_fn bean
impl ItemParser<ItemFn> for ItemFnParser {
    fn parse_item(parse_container: &mut ParseContainer, item_fn: &mut ItemFn, path_depth: Vec<String>) {
        if !Self::is_bean(&item_fn.attrs) {
            return;
        }
        Self::item_fn_parse(item_fn.clone())
            .filter(|fn_found|
                fn_found.fn_type.as_ref().is_some()
                    && fn_found.fn_type.as_ref().unwrap().get_inner_type().is_some()
            )
            .map(|fn_found| Self::add_fn_add_bean(parse_container, &item_fn, path_depth.clone(), &fn_found))
            .or_else(|| {
                log_message!("Could not set fn type for fn named: {}", SynHelper::get_str(item_fn.sig.ident.clone()).as_str());
                None
            });
    }
}

impl ItemFnParser {

    pub fn get_fn_id(fn_type: &FunctionType) -> String {
        fn_type.item_fn.sig.ident.to_string()
    }

    pub fn get_bean(item_fn: &ItemFn, factory_fn: &ModulesFunctions, id: String) -> BeanDefinition {
        BeanDefinition {
            qualifiers: ParseUtil::get_qualifiers(&item_fn.attrs),
            struct_type: factory_fn.fn_found.fn_type.as_ref()
                .map(|bean_path| bean_path.get_inner_type())
                .flatten(),
            struct_found: None,
            traits_impl: vec![],
            enum_found: None,
            deps_map: vec![],
            id,
            path_depth: factory_fn.path.clone(),
            profile: get_profiles(&item_fn.attrs),
            ident: Some(factory_fn.fn_found.item_fn.sig.ident.clone()),
            fields: vec![],
            bean_type: Some(factory_fn.fn_found.bean_type.clone()),
            mutable: false,
            factory_fn: Some(factory_fn.clone()),
            declaration_generics: None
        }
    }

    pub fn parse_fn_arg_generics(item_fn: &ItemFn) -> Generics {
        let generics = item_fn.sig.generics.clone();
        let mut g = Generics::default();
        let output_tys = Self::output_tys(item_fn);
        get_all_generic_ty_bounds(&generics)
            .into_iter().filter(|(k, v)| k.generic_param.is_some())
            .filter(|(k, v)| output_tys.iter()
                .any(|o| o.to_token_stream().to_string().as_str() == k.generic_param.as_ref().to_token_stream().to_string().as_str())
            )
            .for_each(|(generic_ty,_)|
                g.params.push(GenericParam::Type(TypeParam::from(generic_ty.generic_param.unwrap())))
            );
        g
    }


    pub fn output_gens(item_fn: &ItemFn, generics: &HashMap<GenericTy, Vec<Option<TokenStream>>>) -> Generics {
        let output_tys = Self::output_tys(item_fn);
        let g = create_new_gens(generics, output_tys);
        g
    }


    fn output_tys(item_fn: &ItemFn) -> Vec<Type> {
        match &item_fn.sig.output {
            ReturnType::Default => vec![],
            ReturnType::Type(arrow, out) => {
                let parsed = BeanDependencyPathParser::parse_type(out.deref().clone());
                /// Find the generic types from the BeanPath parts, and add them to the return Generics.
                if parsed.is_some() {
                    let mut parsed = parsed.unwrap();
                     return parsed.path_segments.iter()
                         .flat_map(|p_s| match p_s {
                             BeanPathParts::GenType {
                                 inner_ty_opt,
                                 gen_type ,
                                 outer_ty_opt,
                                 ident
                             } => inner_ty_opt
                                 .clone()
                                 .into_iter()
                                 .collect::<Vec<_>>(),
                             _ => vec![]
                         })
                         .collect();
                }
                vec![]
            }
        }
    }

    pub fn get_bean_type(fn_found: &ItemFn) -> Option<BeanPath> {
        match &fn_found.sig.output {
            ReturnType::Default => {
                None
            }
            ReturnType::Type(_, ty) => {
                match ty.deref().clone() {
                    Type::Path(type_path) => {
                        info!("Retrieving bean type");
                        let bean_dep_output_path = BeanDependencyPathParser::parse_type_path(type_path);
                        if bean_dep_output_path.get_inner_type_id().contains("dyn") {
                            panic!("Factory function cannot return abstract type!");
                        }
                        Some(bean_dep_output_path)
                    }
                    _ => {
                        None
                    }
                }
            }
        }
    }

    pub fn item_fn_parse(fn_found: ItemFn) -> Option<FunctionType> {
        Self::get_fn_type(&fn_found, &fn_found.attrs, Self::get_bean_type(&fn_found))
    }

    fn get_fn_type(fn_found: &ItemFn, attr: &Vec<Attribute>, type_ref: Option<BeanPath>) -> Option<FunctionType> {
        ParseUtil::get_singleton_names(&attr)
            .map(|p| (p, BeanType::Singleton))
            .or(ParseUtil::get_prototype_names(&attr).map(|p| (p, BeanType::Prototype)))
            .map(|bean_type_names| {
                let gens = &fn_found.sig.generics;
                let ty_bounds = get_all_generic_ty_bounds(gens);
                FunctionType {
                    item_fn: fn_found.clone(),
                    qualifiers: bean_type_names.0,
                    profiles: ParseUtil::get_profile(&attr),
                    fn_type: type_ref.clone(),
                    bean_type: bean_type_names.1,
                    args: Self::get_injectable_args(fn_found),
                    output_generics: Self::output_gens(fn_found, &ty_bounds).into(),
                    fn_arg_generics: Self::fn_arg_gens(&fn_found, &ty_bounds).into(),
                }
            })
    }

    fn fn_arg_gens(
        fn_args: &ItemFn,
        generics: &HashMap<GenericTy, Vec<Option<TokenStream>>>
    ) -> Generics {
        let all_args = fn_args.sig.inputs.iter()
            .flat_map(|fn_arg| {
                match fn_arg {
                    FnArg::Receiver(_) => {
                        vec![]
                    }
                    FnArg::Typed(value) => {
                        BeanDependencyPathParser::parse_type(value.ty.deref().clone())
                            .into_iter().flat_map(|pt| pt.get_inner_type().into_iter())
                            .collect::<Vec<Type>>()
                    }
                }
            }).collect::<Vec<Type>>();

        create_new_gens(generics, all_args)

    }

    fn get_injectable_args(fn_args: &ItemFn) -> Vec<(Ident, BeanPath, Option<String>, PatType)> {
        fn_args.sig.inputs.iter().flat_map(|fn_arg| {
            match fn_arg {
                FnArg::Receiver(_) => {
                    vec![]
                }
                FnArg::Typed(value) => {

                    let qualifier = SynHelper::get_attr_from_vec(
                        &value.attrs,
                        &vec!["qualifier"],
                    );

                    SynHelper::get_fn_arg_ident_type(value)
                        .map(|s| {
                            BeanDependencyPathParser::parse_type(value.ty.deref().clone())
                                .map(|type_path| vec![(
                                    s.0,
                                    type_path,
                                    qualifier,
                                    value.clone()
                                )])
                                .or(Some(vec![]))
                                .unwrap()
                        })
                        .or(Some(vec![]))
                        .unwrap()
                }
            }
        }).collect::<Vec<(Ident, BeanPath, Option<String>, PatType)>>()
    }

    pub(crate) fn get_fn_for_qualifier(
        fns: &HashMap<String, ModulesFunctions>,
        qualifier: &Option<String>,
        type_of: &Option<&Type>,
    ) -> Option<ModulesFunctions> {
        qualifier.as_ref()
            .map(|qualifier_to_match|
                fns.iter()
                    .filter(|fn_to_check| fn_to_check.1.fn_found
                        .qualifiers
                        .iter()
                        .any(|qual| qualifier_to_match == qual)
                    )
                    .next()
                    .map(|f| f.1.clone())
            )
            .flatten()
            .or_else(|| Self::get_fn_type_by_type(fns, type_of))
    }

    pub(crate) fn get_fn_type_by_type(fns: &HashMap<String, ModulesFunctions>, type_of: &Option<&Type>) -> Option<ModulesFunctions> {
        let mut next = type_of
            .as_ref()
            .map(|type_to_check| Self::filter_modules_fn_by_type_of(fns, type_to_check))
            .or(Some(vec![]))
            .unwrap();
        next.pop()
    }

    fn filter_modules_fn_by_type_of(fns: &HashMap<String, ModulesFunctions>, type_to_check: &Type) -> Vec<ModulesFunctions> {
        let type_of_str = type_to_check.to_token_stream().to_string();
        fns.iter()
            .map(|f| f.1)
            .filter(|f| Self::does_fn_type_match_str(type_of_str.as_str(), f))
            .map(|fn_type| fn_type.clone())
            .collect::<Vec<ModulesFunctions>>()
    }

    fn does_fn_type_match_str(type_of_str: &str, f: &&ModulesFunctions) -> bool {
        f.fn_found.fn_type.as_ref()
            .map(|t| {
                t.get_inner_type()
                    .map(|i| i.to_token_stream().to_string().as_str() == type_of_str)
                    .or(Some(false)).unwrap()
            })
            .or(Some(false))
            .unwrap()
    }

    fn add_fn_add_bean(parse_container: &mut ParseContainer, item_fn: &&mut ItemFn, mut path: Vec<String>, fn_found: &FunctionType) {
        let fn_id = Self::get_fn_id(&fn_found);
        let modules_fn = ModulesFunctions { fn_found: fn_found.clone(), path, id: fn_id.clone() };
        if parse_container.injectable_types_builder.contains_key(&fn_id) {
            parse_container.injectable_types_builder
                .get_mut(&fn_id)
                .map(|bean| bean.factory_fn = Some(modules_fn.clone()));
        } else {
            log_message!("Adding factory fn bean {} to parse container.", SynHelper::get_str(&item_fn.sig.ident));
            parse_container.injectable_types_builder
                .insert(fn_id.clone(), Self::get_bean(&item_fn, &modules_fn, fn_id.clone()));
        }
        parse_container.fns.insert(fn_id.clone(), modules_fn);
    }
}
