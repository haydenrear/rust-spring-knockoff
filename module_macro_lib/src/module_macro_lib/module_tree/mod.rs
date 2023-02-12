use lazy_static::lazy_static;
use std::any::{Any, TypeId};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, LinkedList};
use std::fmt::{Debug, Formatter, Pointer};
use std::ops::Deref;
use std::ptr::slice_from_raw_parts;
use std::sync::{Arc, Mutex};
use syn::{parse_macro_input, DeriveInput, Data, Fields, Field, Item, ItemMod, ItemStruct, FieldsNamed, FieldsUnnamed, ItemImpl, ImplItem, ImplItemMethod, parse_quote, parse, Type, ItemTrait, Attribute, ItemFn, Path, TraitItem, Lifetime, TypePath, QSelf, TypeArray, ItemEnum};
use syn::__private::str;
use syn::parse::Parser;
use syn::spanned::Spanned;
use syn::{
    LitStr,
    Token,
    Ident,
    token::Paren,
};
use quote::{quote, format_ident, IdentFragment, ToTokens, quote_token, TokenStreamExt};
use syn::Data::Struct;
use syn::token::{Bang, For, Token};
use crate::module_macro_lib::parse_container::{ParseContainer};

#[derive(Clone)]
pub struct Bean {
    pub struct_type: Option<Type>,
    pub struct_found: Option<ItemStruct>,
    pub traits_impl: Vec<AutowireType>,
    pub enum_found: Option<ItemEnum>,
    pub attr: Vec<Attribute>,
    // A reference to another DepImpl - the id is the Type.
    pub deps_map: Vec<DepType>,
    pub id: String,
    pub profile: Vec<Profile>,
    pub ident: Option<Ident>,
    pub fields: Vec<Fields>,
    pub bean_type: Option<BeanType>
}

#[derive(Clone)]
pub enum BeanDefinitionType {
    Abstract {
        bean: Bean,
        dep_type: AutowireType
    }, Concrete {
        bean: Bean
    }
}

#[derive(Clone)]
pub struct BeanPath {
    pub(crate) path_segments: Vec<BeanPathParts>
}

#[derive(Clone)]
pub enum BeanPathParts {
    ArcType {
        arc_inner_types: Type
    },
    FnType {
        input_types: Vec<Type>,
        return_type: Option<Type>
    },
    QSelfType {
        q_self: Type
    },
    BindingType {
        associated_type: Type
    },
    GenType {
        inner: Type
    }
}

/**
Will be annotated with #[bean] and #[singleton], #[prototype] as provided factory functions.
 **/
pub struct ModulesFunctions {
    pub fn_found: FunctionType
}

#[derive(Clone)]
pub enum FunctionType {
    Singleton(ItemFn, Option<String>, Option<Type>),
    Prototype(ItemFn, Option<String>, Option<Type>)
}

impl Debug for FunctionType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

#[derive(Clone)]
pub struct AutowireType {
    pub item_impl: ItemImpl,
    pub profile: Vec<Profile>
}

#[derive(Clone, Eq, Ord, PartialOrd, PartialEq, Hash, Debug)]
pub struct Profile {
    pub(crate) profile: String,
}

impl Default for Profile {
    fn default() -> Self {
        Self {
            profile: "DefaultProfile".to_string()
        }
    }
}

#[derive(Clone)]
pub struct DepType {
    pub bean_info: AutowiredField,
    pub lifetime: Option<Lifetime>,
    pub bean_type: Option<BeanType>,
    pub array_type: Option<TypeArray>,
    pub bean_type_path: Option<BeanPath>
}

#[derive(Clone, Debug)]
pub enum BeanType {
    // contains the identifier and the qualifier as string
    Singleton(BeanDefinition, Option<FunctionType>),
    Prototype(BeanDefinition, Option<FunctionType>)
}


#[derive(Clone)]
pub struct BeanDefinition {
    pub qualifier: Option<String>,
    pub bean_type_type: Option<Type>,
    pub bean_type_ident: Option<Ident>,
}

impl Debug for BeanDefinition {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut debug_struct = f.debug_struct("bean_type");
        self.qualifier.as_ref().and_then(|q| {
            debug_struct.field("qualifier", &q.as_str());
            None::<String>
        });
        self.bean_type_ident.as_ref().and_then(|q| {
            debug_struct.field("bean_type_ident", &q.to_token_stream().to_string().as_str());
            None::<Ident>
        });
        self.bean_type_type.as_ref().and_then(|q| {
            debug_struct.field("bean_type_type", &q.to_token_stream().to_string().as_str());
            None::<Type>
        });
        debug_struct.finish()
    }
}

#[derive(Clone)]
pub struct AutowiredField {
    pub qualifier: Option<String>,
    pub lazy: bool,
    pub field: Field,
    pub type_of_field: Type
}

impl Default for Bean {
    fn default() -> Self {
        Self {
            struct_type: None,
            struct_found: None,
            traits_impl: vec![],
            attr: vec![],
            enum_found: None,
            deps_map: vec![],
            id: String::default(),
            profile: vec![],
            ident: None,
            fields: vec![],
            bean_type: None
        }
    }
}

pub struct Trait {
    pub trait_type: Option<ItemTrait>,
}

impl Trait {
    pub fn new(trait_type: ItemTrait) -> Self {
        Self {
            trait_type: Some(trait_type)
        }
    }
}

#[derive(Clone, Eq, Ord, PartialOrd, PartialEq, Hash)]
pub struct InjectableTypeKey {
    pub underlying_type: String,
    pub impl_type: Option<String>,
    pub profile: Vec<Profile>
}

impl Default for Trait {
    fn default() -> Self {
        Self {
            trait_type: None
        }
    }
}

pub enum GetBeanResultError {
    BeanNotInContext, BeanDependenciesNotSatisfied
}

pub struct TestFieldAdding;

// A way to edit fields of structs - probably only possible to do through attributes..
impl TestFieldAdding {
    pub fn process(&self, struct_item: &mut ItemStruct) {
        match &mut struct_item.fields {
            Fields::Named(ref mut fields_named) => {
                fields_named.named.push(
                    Field::parse_named.parse2(quote!(
                        pub a: String
                    ).into()).unwrap()
                )
            }
            Fields::Unnamed(ref mut fields_unnamed) => {}
            _ => {}
        }
    }
}
