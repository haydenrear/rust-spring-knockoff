use lazy_static::lazy_static;
use std::any::{Any, TypeId};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, LinkedList};
use std::ops::Deref;
use std::ptr::slice_from_raw_parts;
use std::sync::{Arc, Mutex};
use syn::{parse_macro_input, DeriveInput, Data, Fields, Field, Item, ItemMod, ItemStruct, FieldsNamed, FieldsUnnamed, ItemImpl, ImplItem, ImplItemMethod, parse_quote, parse, Type, ItemTrait, Attribute, ItemFn, Path, TraitItem, Lifetime, TypePath, QSelf};
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
use crate::module_macro_lib::module_container::ModuleContainer;

pub struct DepImpl {
    pub struct_type: Option<Type>,
    pub struct_found: Option<ItemStruct>,
    pub traits_impl: Vec<ItemImpl>,
    pub attr: Vec<Attribute>,
    // A reference to another DepImpl - the id is the Type.
    pub deps_map: Vec<DepType>,
    pub id: String,
    pub profile: Vec<Profile>,
    pub ident: Option<Ident>
}

pub struct Profile {
    profile: Vec<String>,
}

#[derive(Clone)]
pub struct DepType {
    pub id: String,
    pub is_ref: bool,
    pub type_found: Type,
    pub ident: Option<Ident>,
    pub dep_path: Path
}

impl Default for DepImpl {
    fn default() -> Self {
        Self {
            struct_type: None,
            struct_found: None,
            traits_impl: vec![],
            attr: vec![],
            deps_map: vec![],
            id: String::default(),
            profile: vec![],
            ident: None
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

impl Default for Trait {
    fn default() -> Self {
        Self {
            trait_type: None
        }
    }
}

impl Default for ModuleContainer {
    fn default() -> Self {
        Self {
            traits: HashMap::new(),
            types: HashMap::new(),
            fns: HashMap::new(),
            profiles: vec![],
        }
    }
}


pub struct ApplicationContainer {
    pub modules: Vec<ModuleContainer>,
}

macro_rules! test_field_add {
    ($tt:tt) => {

    }
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
