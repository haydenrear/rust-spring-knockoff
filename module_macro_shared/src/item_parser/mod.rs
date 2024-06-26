use std::any::Any;
use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::thread::available_parallelism;
use quote::ToTokens;
use syn::{Attribute, Fields, GenericParam, Generics, ImplGenerics, ImplItem, Item, ItemEnum, ItemFn, ItemImpl, ItemMod, ItemStruct, ItemTrait, parse2, Type, TypeGenerics, TypeParam, TypeParamBound, WherePredicate};
use codegen_utils::syn_helper::SynHelper;
use item_impl_parser::ItemImplParser;
use crate::module_macro_shared_codegen::FieldAugmenter;

use crate::module_tree::Trait;
use crate::parse_container::ParseContainer;

use crate::bean::BeanDefinition;
use crate::dependency::DependencyDescriptor;
use crate::functions::ModulesFunctions;
use crate::profile_tree::ProfileBuilder;
use crate::util::ParseUtil;

use paste::item;
use knockoff_logging::*;
use lazy_static::lazy_static;
use std::sync::Mutex;
use proc_macro2::{Ident, TokenStream};
use quote::__private::ext::RepToTokensExt;
use codegen_utils::project_directory;
use crate::{BuildParseContainer, ItemModifier, logger_lazy, ModuleParser, ParseContainerItemUpdater, ParseContainerModifier, ProfileTreeFinalizer};
import_logger!("item_parser.rs");


pub mod item_impl_parser;
pub mod item_enum_parser;
pub mod item_struct_parser;
pub mod item_mod_parser;
pub mod item_trait_parser;
pub mod item_fn_parser;

pub trait ItemParser<T: ToTokens> {
    fn parse_item<
        ParseContainerItemUpdaterT: ParseContainerItemUpdater,
        ItemModifierT: ItemModifier,
        ParseContainerModifierT: ParseContainerModifier,
        BuildParseContainerT: BuildParseContainer,
        ParseContainerFinalizerT: ProfileTreeFinalizer,
    >(
        program_src: &PathBuf,
        parse_container: &mut ParseContainer,
        item: &mut T,
        path_depth: Vec<String>,
        module_parser: &mut ModuleParser<
            ParseContainerItemUpdaterT,
            ItemModifierT,
            ParseContainerModifierT,
            BuildParseContainerT,
            ParseContainerFinalizerT
        >
    );
    fn is_bean(attrs: &Vec<Attribute>) -> bool {
        ParseUtil::does_attr_exist(&attrs, &ParseUtil::get_qualifier_attr_names())
    }
}

fn get_profiles(attrs: &Vec<Attribute>) -> Vec<ProfileBuilder> {
    let mut profiles = SynHelper::get_attr_from_vec(attrs, &vec!["profile"])
        .map(|profile| profile.split(",").map(|s| s.to_string()).collect::<Vec<String>>())
        .or(Some(vec![]))
        .unwrap()
        .iter()
        .map(|profile| ProfileBuilder {profile: profile.replace(" ", "")})
        .collect::<Vec<ProfileBuilder>>();
    profiles.push(ProfileBuilder::default());
    profiles
}

pub struct GenericTy {
    pred_type: Option<Type>,
    generic_param: Option<Ident>
}

impl Hash for GenericTy {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write(self.pred_type.to_token_stream().to_string().as_bytes());
    }
}

impl Debug for GenericTy {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("Generic TY: ")?;
        f.write_str(SynHelper::get_str(&self.pred_type).as_str())?;
        Ok(())
    }
}

pub fn get_all_generic_ty_bounds(generics: &Generics) -> HashMap<GenericTy, Vec<Option<TokenStream>>> {
    let mut out = HashMap::new();
    let _ = generics.where_clause.as_ref().iter()
        .flat_map(|w| w.predicates.iter())
        .map(|pred| {
            // match pred {
            //     WherePredicate::Type(ty_value) => {
            //         info!("Adding where predicate {:?}", SynHelper::get_str(ty_value));
            //         out.insert(GenericTy {
            //             pred_type: Some(ty_value.bounded_ty.clone()),
            //             generic_param: None
            //         }, vec![]);
            //     }
            //     WherePredicate::Lifetime(_) => {}
            //     WherePredicate::Eq(_) => {}
            // }
        });
    // let _ = generics.params.iter().map(|p| {
    //     match p {
    //         /// Weird this is the same as below but doesn't return the same thing.
    //         GenericParam::Type(ty) => {
    //             info!("Adding ty {:?}", SynHelper::get_str(ty));
    //             let _ = parse2::<Type>(ty.ident.to_token_stream())
    //                 .map(|parsed_ty| {
    //                     add_bounds_with_def(&mut out, ty, parsed_ty, true);
    //                 });
    //         }
    //         GenericParam::Lifetime(_) => {}
    //         GenericParam::Const(_) => {}
    //     }
    // });
    // generics.type_params()
    //     .into_iter()
    //     .for_each(|ty_param| {
    //         let _ = parse2(ty_param.ident.to_token_stream())
    //             .map(|ty_value| add_bounds_with_def(
    //                 &mut out, ty_param, ty_value)
    //             );
    //     });
    info!("Parsed all generic tys for generic: {:?}: {:?}", SynHelper::get_str(&generics),&out);
    out
}


pub(crate) fn create_new_gens(generics: &HashMap<GenericTy, Vec<Option<TokenStream>>>, output_tys: Vec<Type>) -> Generics {
    let mut g = Generics::default();
    generics.into_iter()
        .filter(|(k, v)| k.generic_param.is_some())
        .filter(|(k, v)| output_tys.iter()
            .any(|o| o.to_token_stream().to_string().as_str() == k.generic_param.as_ref().to_token_stream().to_string().as_str())
        )
        .for_each(|(generic_ty, _)|
            g.params.push(GenericParam::Type(TypeParam::from(generic_ty.generic_param.clone().unwrap())))
        );
    g
}

// pub(crate) fn add_bounds_with_def(mut out: &mut HashMap<GenericTy, Vec<Option<TokenStream>>>,
//                        ty: &TypeParam, parsed_ty: Ident) {
//     if ty.bounds.len() == 0 {
//         collection_util::add_to_multi_value(
//             &mut out, None, GenericTy{
//                 generic_param: Some(parsed_ty),
//                 pred_type: None
//             });
//     } else {
//         add_bounds(&mut out, ty, parsed_ty);
//     }
// }
//
// pub(crate) fn add_bounds(mut out: &mut HashMap<GenericTy, Vec<Option<TokenStream>>>, ty_param: &TypeParam,
//               ty_value: Ident) {
//     ty_param.bounds.iter().for_each(|bound| {
//         match bound {
//             TypeParamBound::Trait(trait_bound) => {
//                 collection_util::add_to_multi_value(
//                     &mut out,
//                     Some(trait_bound.path.to_token_stream()),
//                     GenericTy {
//                         generic_param: Some(ty_value.clone()),
//                         pred_type: None
//                     }
//                 );
//             }
//             TypeParamBound::Lifetime(_) => {}
//         }
//     });
// }
