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
use module_macro_shared::parse_container::ParseContainer;

use_logging!();
initialize_log!();

use crate::module_macro_lib::logging::executor;
use crate::module_macro_lib::logging::StandardLoggingFacade;


#[derive(Clone)]
pub struct BeanDefinition {
    pub qualifier: Option<String>,
    pub bean_type_type: Option<Type>,
    pub bean_type_ident: Option<Ident>,
    pub bean_type: BeanType
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




