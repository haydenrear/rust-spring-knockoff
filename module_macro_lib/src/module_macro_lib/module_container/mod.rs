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
use crate::module_macro_lib::module_tree::{DepImpl, Trait, Profile, DepType};


/**
Will be annotated with #[bean] and #[singleton], #[prototype] as provided factory functions.
 **/
pub struct ModulesFunctions {
    pub fn_found: ItemFn,
}

pub struct ModuleContainer {
    pub types: HashMap<String, DepImpl>,
    pub traits: HashMap<String, Trait>,
    pub fns: HashMap<String, ModulesFunctions>,
    pub profiles: Vec<Profile>,
}

impl ModuleContainer {
    /**
    Generate the token stream from the created ModuleContainer tree.
     **/
    pub fn to_token_stream(&self) -> proc_macro2::TokenStream {
        let mut token = quote! {};
        for token_type in &self.types {
            println!("Implementing container for {} if is not none and implements Default.", token_type.1.id.clone());
            if token_type.1.struct_type.is_some() {

                let struct_type =  token_type.1.struct_type.clone()
                    .unwrap();

                println!("Implementing container for {}.", struct_type.to_token_stream().to_string().clone());

                let field_types = token_type.1.deps_map
                    .clone().iter()
                    .map(|d| d.type_found.clone())
                    .collect::<Vec<Type>>();

                let identifiers = token_type.1.deps_map
                    .clone().iter()
                    .flat_map(|d| {
                        match &d.ident {
                            None => {
                                vec![]
                            }
                            Some(identifier) => {
                                vec![identifier.clone()]
                            }
                        }
                    })
                    .collect::<Vec<Ident>>();

                let this_struct_impl = quote! {

                    impl Container<#struct_type> for AppContainer {
                        fn get_create(&self) -> Component<#struct_type> {
                            let this_component = <Component<#struct_type>>::new();
                            this_component
                        }
                    }

                    impl Component<#struct_type> {
                        fn new() -> Self {
                            let mut inner = #struct_type::default();
                            #(
                                inner.#identifiers = AppContainer::get_create::<#field_types>();
                            )*
                            Self {
                                inner: Some(inner)
                            }
                        }
                    }
                };

                token.append_all(this_struct_impl);

            }
        }

        token
    }

    /**
    Add the struct and the impl from the ItemImpl
     **/
    pub fn create_update_impl(&mut self, item_impl: &mut ItemImpl) {
        let id = item_impl.self_ty.to_token_stream().to_string().clone();
        &mut self.types.get_mut(&id)
            .map(|struct_impl: &mut DepImpl| {
                struct_impl.traits_impl.push(item_impl.clone());
            })
            .or_else(|| {
                let impl_found = DepImpl {
                    struct_type: Some(item_impl.self_ty.deref().clone()),
                    struct_found: None,
                    traits_impl: vec![item_impl.clone()],
                    attr: vec![],
                    deps_map: vec![],
                    id: id.clone(),
                    profile: vec![],
                    ident: None
                };
                self.types.insert(id, impl_found);
                None
            });
    }

    pub fn add_item_struct(&mut self, item_impl: &mut ItemStruct) {
        println!("adding type with name {}", item_impl.ident.clone().to_token_stream().to_string());
        &mut self.types.get_mut(&item_impl.ident.to_string().clone())
            .map(|struct_impl: &mut DepImpl| {
                struct_impl.struct_found = Some(item_impl.clone());
            })
            .or_else(|| {
                let mut impl_found = DepImpl {
                    struct_type: None,
                    struct_found: Some(item_impl.clone()),
                    traits_impl: vec![],
                    attr: vec![],
                    deps_map: vec![],
                    id: item_impl.ident.clone().to_string(),
                    profile: vec![],
                    ident: Some(item_impl.ident.clone())
                };
                self.types.insert(item_impl.ident.to_string().clone(), impl_found);
                None
            });
        self.set_deps(item_impl);
    }

    pub fn create_update_trait(&mut self, trait_found: &mut ItemTrait) {
        if !self.traits.contains_key(&trait_found.ident.to_string().clone()) {
            self.traits.insert(trait_found.ident.to_string().clone(), Trait::new(trait_found.clone()));
        } else {
            println!("Contained trait already!");
        }
    }

    pub fn set_deps(&mut self, item_impl: &mut ItemStruct) {
        match item_impl.fields.clone() {
            Fields::Named(fields_named) => {
                fields_named.named.iter().for_each(|field: &Field| {
                    field.clone().ident.map(|ident: Ident| {
                        println!("found field {}.", ident.to_string().clone());
                    });
                    println!("{} is the field type!", field.ty.to_token_stream().clone());
                    self.match_ty_add_dep_recursive(item_impl, field.ty.clone(), false);
                });
            }
            Fields::Unnamed(unnamed_field) => {}
            _ => {}
        };
    }

    /**
    Adds the field to the to the tree as a dependency.
    //TODO: need to recursively update tree for references, arrays, etc arbitrarily deep.
    **/
    pub fn match_ty_add_dep_recursive(&mut self, item_impl: &mut ItemStruct, field: Type, is_ref: bool) {
        match field.clone() {
            Type::Array(arr) => {
                println!("found array type {}.", arr.to_token_stream().to_string().clone());
            }
            Type::BareFn(bare_fn) => {
                println!("found bare fn type {}.", bare_fn.to_token_stream().to_string().clone());
            }
            Type::Group(grp) => {
                println!("found group type {}.", grp.to_token_stream().to_string().clone());
            }
            Type::ImplTrait(grp) => {
                println!("found impl trait type {}.", grp.to_token_stream().to_string().clone());
            }
            Type::Infer(grp) => {
                println!("found infer type {}.", grp.to_token_stream().to_string().clone());
            }
            Type::Macro(grp) => {
                println!("found macro type {}.", grp.to_token_stream().to_string().clone());
            }
            Type::Never(grp) => {
                println!("found never type {}.", grp.to_token_stream().to_string().clone());
            }
            Type::Paren(grp) => {
                println!("found paren type {}.", grp.to_token_stream().to_string().clone());
            }
            Type::Path(path) => {
                println!("Adding path: {}.", path.path.to_token_stream().to_string().clone());
                path.qself.map(|q_self| {
                    println!("Asserting that {} and {} are the same.", q_self.ty.clone().to_token_stream().clone(), field.clone().to_token_stream().to_string().clone());
                });
                self.add_type_dep(item_impl, path.path.clone(), false, field.clone(), item_impl.ident.clone());
            }
            Type::Ptr(grp) => {
                println!("found ptr type {}.", grp.to_token_stream().to_string().clone());
            }
            Type::Reference(reference_found) => {
                let ref_type = reference_found.elem.clone();
                println!("{} is the ref type", ref_type.to_token_stream());
                self.match_ty_add_dep_recursive(item_impl, ref_type.clone().deref().clone(), true)
            }
            Type::Slice(grp) => {
                println!("found slice type {}.", grp.to_token_stream().to_string().clone());
            }
            Type::TraitObject(grp) => {
                println!("found trait object type {}.", grp.to_token_stream().to_string().clone());
            }
            Type::Tuple(grp) => {
                println!("found tuple type {}.", grp.to_token_stream().to_string().clone());
            }
            Type::Verbatim(grp) => {
                println!("found verbatim type {}.", grp.to_token_stream().to_string().clone());
            }
            _ => {}
        };
    }

    pub fn match_ty_recursive_get_dependency(
        &mut self,
        type_to_resolve: &Type,
    ) -> DepImpl {
        match type_to_resolve {
            Type::Array(arr) => {
                println!("found field hello");
            }
            Type::BareFn(_) => {
                println!("found field hello");
            }
            Type::Group(_) => {
                println!("found field hello");
            }
            Type::ImplTrait(_) => {
                println!("found field hello");
            }
            Type::Infer(_) => {
                println!("found field hello");
            }
            Type::Macro(_) => {
                println!("found field hello");
            }
            Type::Never(_) => {
                println!("found field hello");
            }
            Type::Paren(_) => {
                println!("HELLO")
            }
            Type::Path(path) => {
                // println!("Adding path: {}.", path.path.to_token_stream().to_string().clone());
            }
            Type::Ptr(_) => {
                println!("found ptr");
            }
            Type::Reference(reference_found) => {
                println!("is the ref type");
            }
            Type::Slice(_) => {
                println!("found field hello");
            }
            Type::TraitObject(_) => {
                println!("found field hello");
            }
            Type::Tuple(_) => {
                println!("found field hello");
            }
            Type::Verbatim(_) => {
                println!("found field hello");
            }
            _ => {}
        };
        DepImpl::default()
    }

    pub fn add_type_dep(&mut self, item_impl: &mut ItemStruct, path: Path, is_ref: bool,
                    type_found: Type, new_item_ident: Ident,
    )
    {
        let type_dep = &type_found.to_token_stream().to_string();
        let contains_key = self.types.contains_key(type_dep);
        let struct_exists = self.types.get_mut(&new_item_ident.to_string().clone()).is_some();
        let id = self.types.get(type_dep)
            .and_then(|struct_found: &DepImpl| Some((struct_found.id.clone())))
            .or(Some(path.to_token_stream().to_string().clone()))
            .unwrap();
        if contains_key && struct_exists && id != String::default() {
            println!("Adding dependency with id {} to struct_impl of name {}", id, &item_impl.ident.to_string().clone());
            self.types.get_mut(&item_impl.ident.to_string().clone())
                .unwrap()
                .deps_map
                .push(DepType {
                    ident: Some(new_item_ident),
                    id,
                    is_ref: is_ref, type_found,
                    dep_path: path.clone()
                });
        } else {
            println!("Could not add dependency {} to struct_impl {}!", id.clone(), item_impl.ident.to_string().clone());
            if !struct_exists {
                println!("Struct impl did not exist in module container.")
            }
            if !contains_key {
                println!("Dependency did not exist in module container.")
            }
        }
    }
}
