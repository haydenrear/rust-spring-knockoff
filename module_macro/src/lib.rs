extern crate proc_macro;

use delegator_macro_rules::types;
use lazy_static::lazy_static;
use proc_macro::{Span, TokenStream};
use std::any::{Any, TypeId};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, LinkedList};
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use syn::{parse_macro_input, DeriveInput, Data, Fields, Field, Item, ItemMod, ItemStruct, FieldsNamed, FieldsUnnamed, ItemImpl, ImplItem, ImplItemMethod, parse_quote, parse, Type, ItemTrait, Attribute, ItemFn, Path, TraitItem};
use syn::__private::str;
use syn::parse::Parser;
use syn::spanned::Spanned;
use rust_spring_macro::module_post_processor::{ModuleFieldPostProcessor, ModuleStructPostProcessor};
use syn::{
    LitStr,
    Token,
    Ident,
    token::Paren,
};

use quote::{quote, format_ident, IdentFragment, ToTokens};
use syn::Data::Struct;
use syn::token::{Bang, For, Token};

#[proc_macro_attribute]
pub fn module_attr(attr: TokenStream, input: TokenStream) -> TokenStream {
    let mut found: Item = parse_macro_input!(input as Item);
    parse_module(found)
}

/// # assumptions
/// 1. all modules are moved in-line, so this attribute macro has access to all modules
/// 2. if the type that is to be injected is declared outside of the module, then there is a function declaration
/// annotated with #[bean] or #[factory] that provides it.
/// 2.1. If it is also annotated with #[transient] or #[prototype] then it
/// is prototype bean and if it's also annotated with #[singleton] then it's singleton bean
/// 3. each module annotated with #[module_attr] creates a container with the name of the module,
/// removing snake case and replacing with camel case and ending with Container.
/// 3.1. The container created by any module with nested modules also contains and delegates to the Containers
/// of those modules if those nested modules are annotated with #[module_attr]
/// # features
/// 1. The container created should be able to get the singleton of any bean marked #[singleton] or
/// create an implementation. This should happen by passing the type_id() to the container.
/// 2. Any dependency should be wired. If the dependency is annotated with #[singleton] then it should
/// add the singleton, and if it is marked #[prototype] then it should create a new one and add it as field.
/// # how
/// 1. parse modules into a tree/map that contains dependencies and Ident as the key.
/// 2. for each type that contains dependencies, move into the tree and recursively implement,
/// moving backwards once finished implementing that one to finish implementing the prev.
/// 2.1 There will be a Lazy<T> type that will set None and keep a record of the ones that are lazy,
/// and go back to add the fn [<get_lazy_[#type]](&self).

// fn create_get_processors(input: TokenStream) -> Vec<&dyn ModuleStructPostProcessor>  {
//     let mut found: Item = parse_macro_input!(input as Item);
//     let impls = parse_get_impls(found);
//     impls.iter().map(|&i| {
//         parse_get_structs(i as Item)
//     })
//     let structs = parse_get_structs(impls);
// }

fn parse_module(mut found: Item) -> TokenStream {
    match &mut found {
        Item::Mod(ref mut struct_found) => {
            let mut container = ModuleContainer::default();
            parse_item_recursive(struct_found, &mut container);
            return quote!(#found).into();
        }
        _ => {
            return quote!(#found).into();
        }
    }
}

fn parse_item_recursive(item_found: &mut ItemMod, module_container: &mut ModuleContainer) {
    item_found.content.iter_mut()
        .flat_map(|mut c| c.1.iter_mut())
        .for_each(|i: &mut Item|  parse_item(i, module_container));
}

/// 1. parse the module into struct impls that contain the struct, all impls
/// 2. iterate through each of struct impls and implement get_create for each of the types
///    for the container that contains.
/// Then, you impl create<StructImpls> for the container for each StructItem add container as
/// field for each struct, and then create a new_inject() for each item that calls the
/// create_get<StructImpl> for each field that is injected.
struct StructImpls {
    struct_type: Option<ItemImpl>,
    struct_found: Option<ItemStruct>,
    traits: Vec<Path>,
    attr:   Vec<Attribute>,
    deps_map: Vec<DepType>,
    id: String
}

#[derive(Clone)]
struct DepType {
    id: String,
    is_ref: bool
}

impl Default for StructImpls {
    fn default() -> Self {
        Self {
            struct_type: None,
            struct_found: None,
            traits: vec![],
            attr: vec![],
            deps_map: vec![],
            id: String::default()
        }
    }
}

struct Trait {
    trait_type: Option<ItemTrait>
}

impl Trait {
    fn new(trait_type: ItemTrait) -> Self {
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

struct ModulesFunctions {
    fn_found: ItemFn
}

struct ModuleContainer {
    types: HashMap<String, StructImpls>,
    traits: HashMap<String, Trait>,
    fns: HashMap<String, ModulesFunctions>,
}

impl ModuleContainer {

    fn create_update_impl(&mut self, item_impl: &mut ItemImpl) {
        println!("adding type with name {}", item_impl.self_ty.clone().to_token_stream().to_string());
        let id = item_impl.self_ty.to_token_stream().to_string().clone();
        &mut self.types.get_mut(&id)
            .map(|struct_impl: &mut StructImpls| {
                get_traits(item_impl)
                    .map(|path| {
                        struct_impl.traits.push(path);
                    });
            })
            .or_else(|| {
                let impl_found = StructImpls {
                    struct_type: Some(item_impl.clone()),
                    struct_found: None,
                    traits: get_traits(item_impl).map(|path| vec![path]).unwrap_or(vec![]),
                    attr: vec![],
                    deps_map: vec![],
                    id: id.clone()
                };
                self.types.insert(id, impl_found);
                None
            });
    }

    fn add_item_struct(&mut self, item_impl: &mut ItemStruct) {
        println!("adding type with name {}", item_impl.ident.clone().to_token_stream().to_string());
        &mut self.types.get_mut(&item_impl.ident.to_string().clone())
            .map(|struct_impl: &mut StructImpls| {
                struct_impl.struct_found = Some(item_impl.clone());
            })
            .or_else(|| {
                let impl_found = StructImpls {
                    struct_type: None,
                    struct_found: Some(item_impl.clone()),
                    traits: vec![],
                    attr: vec![],
                    deps_map: vec![],
                    id: item_impl.ident.to_string(),
                };
                self.types.insert(item_impl.ident.to_string().clone(), impl_found);
                None
            });
        self.get_deps(item_impl);
    }

    fn create_update_trait(&mut self, trait_found: &mut ItemTrait) {
        if !self.traits.contains_key(&trait_found.ident.to_string().clone()) {
            self.traits.insert(trait_found.ident.to_string().clone(), Trait::new(trait_found.clone()));
        } else {
            println!("Contained trait already!");
        }
    }

    // Called after this ItemStruct has been added.
    fn get_deps(&mut self, item_impl: &mut ItemStruct) {
        println!("hello");
        match item_impl.fields.clone() {
            Fields::Named(fields_named) => {
                fields_named.named.iter().for_each(|field: &Field| {
                    field.clone().ident.map(|ident: Ident| {
                        println!("found field {}.", ident.to_string().clone());
                    });
                    println!("{} is the field type!", field.ty.to_token_stream().clone());
                    self.match_ty_recursive(item_impl, field.ty.clone());
                });
            }
            _ => {
            }
        };
    }

    fn match_ty_recursive(&mut self, item_impl: &mut ItemStruct, field: Type) {
        match field.clone() {
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
                println!("Adding path: {}.", path.path.to_token_stream().to_string().clone());
                self.add_type(item_impl, path.path.clone(), false, field.clone(), item_impl.ident.clone());
            }
            Type::Ptr(_) => {
                println!("found ptr");
            }
            Type::Reference(reference_found) => {
                let ref_type = reference_found.elem.clone();
                println!("{} is the ref type", ref_type.to_token_stream());

                self.match_ty_recursive(item_impl, ref_type.clone().deref().clone())
                // TODO: match type recursively
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
    }

    fn add_type(
        &mut self,
        item_impl: &mut ItemStruct,
        path: Path,
        is_ref: bool,
        type_found: Type,
        new_item_ident: Ident,
    ) {
        println!("{} IS THE PATH!", path.to_token_stream().to_string().clone());
        let type_dep = &type_found.to_token_stream().to_string();
        let contains_key = self.types.contains_key(type_dep);
        let struct_exists = self.types.get_mut(&new_item_ident.to_string().clone()).is_some();
        let id = self.types.get(type_dep)
            .and_then(|struct_found: &StructImpls| Some(struct_found.id.clone()))
            .or(Some(path.to_token_stream().to_string().clone()))
            .unwrap();
        if contains_key && struct_exists && id != String::default() {
            self.types.get_mut(
                &item_impl.ident.to_string().clone()
            )
                .unwrap()
                .deps_map
                .push(
                    DepType{
                        id, is_ref: is_ref
                    }
                );
        } else {
            println!("Could not add dependency to struct_impl!");
            if !struct_exists {
                println!("Struct impl did not exist in module container.")
            }
            if !contains_key {
                println!("Dependency did not exist in module container.")
            }
        }
    }

}


fn get_traits(item_impl: &mut ItemImpl) -> Option<Path> {
    item_impl.trait_.clone()
        .and_then(|item_impl_found| {
            Some(item_impl_found.1)
        })
        .or_else(|| None)
}


impl Default for ModuleContainer {
    fn default() -> Self {
        Self {
            traits: HashMap::new(),
            types: HashMap::new(),
            fns: HashMap::new()
        }
    }
}

struct Component<T> {
    inner: Option<T>
}

trait Container {
    fn get_create<T>(&self, type_id: String) -> Component<T>
    where Self: Sized;
}

struct ApplicationContainer {
    modules: Vec<ModuleContainer>
}

fn parse_item(i: &mut Item, mut module_container: &mut ModuleContainer) {
    match i {
        Item::Const(_) => {
        }
        Item::Enum(_) => {}
        Item::ExternCrate(_) => {}
        Item::Fn(_) => {}
        Item::ForeignMod(_) => {}
        Item::Impl(impl_found) => {
            module_container.create_update_impl(impl_found);
        }
        Item::Macro(_) => {}
        Item::Macro2(_) => {}
        Item::Mod(ref mut module) => {
            println!("Found module with name {} !!!", module.ident.to_string().clone());
            parse_item_recursive(module, module_container);
        }
        Item::Static(_) => {}
        Item::Struct(ref mut item_struct) => {
            let f = ImplRunningPostProcessor{};
            f.process(item_struct);
            module_container.add_item_struct(item_struct);
            println!("Found struct with name {} !!!", item_struct.ident.to_string().clone());
        }
        Item::Trait(trait_created) => {
            println!("Trait created: {}", trait_created.ident.clone().to_string());
            module_container.create_update_trait(trait_created);
        }
        Item::TraitAlias(_) => {}
        Item::Type(_) => {
            println!("Item type found!")
        }
        Item::Union(_) => {}
        Item::Use(_) => {}
        Item::Verbatim(_) => {}
        _ => {}
    }




    struct ImplRunningPostProcessor;
    // A way to edit fields of structs - probably only possible to do through attributes..
    impl ImplRunningPostProcessor {
        fn process(&self, struct_item: &mut ItemStruct) {
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

}