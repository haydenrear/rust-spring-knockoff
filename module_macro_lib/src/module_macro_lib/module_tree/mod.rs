use lazy_static::lazy_static;
use std::any::{Any, TypeId};
use std::cmp::Ordering;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, LinkedList};
use std::fmt::{Debug, Formatter, Pointer};
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use syn::{Attribute, Block, Data, DeriveInput, Field, Fields, FieldsNamed, FieldsUnnamed, ImplItem, ImplItemMethod, Item, ItemEnum, ItemFn, ItemImpl, ItemMod, ItemStruct, ItemTrait, Lifetime, parse, parse_macro_input, parse_quote, Path, QSelf, Stmt, TraitItem, Type, TypeArray, TypePath};
use syn::__private::str;
use syn::parse::Parser;
use syn::spanned::Spanned;
use syn::{Ident, LitStr, Token, token::Paren};
use quote::{format_ident, IdentFragment, quote, quote_token, TokenStreamExt, ToTokens};
use syn::Data::Struct;
use syn::token::{Bang, For, Token};
use module_macro_shared::aspect::{AspectInfo, MethodAdviceChain};
use codegen_utils::syn_helper::SynHelper;
use knockoff_logging::{initialize_log, use_logging};
use module_macro_codegen::aspect::MethodAdviceAspectCodegen;
use module_macro_shared::bean::{Bean, BeanPath, BeanPathParts, BeanType};
use module_macro_shared::dependency::{AutowiredField, AutowireType, DepType};
use module_macro_shared::profile_tree::ProfileBuilder;
use crate::module_macro_lib::parse_container::ParseContainer;

use_logging!();
initialize_log!();

use crate::module_macro_lib::logging::executor;
use crate::module_macro_lib::logging::StandardLoggingFacade;

/**
Will be annotated with #[bean] and #[singleton], #[prototype] as provided factory functions.
 **/
pub struct ModulesFunctions {
    pub fn_found: FunctionType,
    pub path: Vec<String>
}

#[derive(Clone)]
pub struct FunctionType {
    pub(crate) item_fn: ItemFn,
    pub(crate) qualifier: Option<String>,
    pub(crate) fn_type: Option<Type>,
    pub(crate) bean_type: BeanType
}


#[derive(Clone)]
pub struct BeanDefinition {
    pub qualifier: Option<String>,
    pub bean_type_type: Option<Type>,
    pub bean_type_ident: Option<Ident>,
    pub bean_type: BeanType
}

#[derive(Default, Clone)]
pub struct Trait {
    pub trait_type: Option<ItemTrait>,
    pub trait_path: Vec<String>
}

impl Trait {
    pub fn new(trait_type: ItemTrait, path: Vec<String>) -> Self {
        Self {
            trait_type: Some(trait_type),
            trait_path: path,
        }
    }
}

#[derive(Clone, Eq, Ord, PartialOrd, PartialEq, Hash)]
pub struct InjectableTypeKey {
    pub underlying_type: String,
    pub impl_type: Option<String>,
    pub profile: Vec<ProfileBuilder>
}

pub enum GetBeanResultError {
    BeanNotInContext, BeanDependenciesNotSatisfied
}




