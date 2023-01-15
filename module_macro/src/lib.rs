extern crate proc_macro;

use delegator_macro_rules::{add, types};
use lazy_static::lazy_static;
use proc_macro::{Span, TokenStream};
use std::any::{Any, TypeId};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, LinkedList};
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use syn::{parse_macro_input, DeriveInput, Data, Fields, Field, Item, ItemMod, ItemStruct, FieldsNamed, FieldsUnnamed, ItemImpl, ImplItem, ImplItemMethod, parse_quote, parse, Type, ItemTrait, Attribute, ItemFn, Path, TraitItem, Lifetime};
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
use quote::{quote, format_ident, IdentFragment, ToTokens, quote_token, TokenStreamExt};
use syn::Data::Struct;
use syn::token::{Bang, For, Token};

#[proc_macro_attribute]
pub fn module_attr(attr: TokenStream, input: TokenStream) -> TokenStream {

    let mut token_stream_builder = TokenStreamBuilder::default();
    let input_found = input.clone();
    token_stream_builder.add_to_tokens(
        write_starting_types()
    );

    let mut found: Item = parse_macro_input!(input_found as Item);
    let additional = parse_module(found);

    token_stream_builder.add_to_tokens(additional);
    token_stream_builder.build()
}

#[derive(Default)]
struct TokenStreamBuilder {
    stream_build: Vec<TokenStream>
}



impl TokenStreamBuilder {

    fn add_to_tokens(&mut self, tokens: TokenStream) {
        self.stream_build.push(tokens);
    }

    fn build(&self) -> TokenStream {
        let mut final_tokens = TokenStream::default();
        self.stream_build.iter().for_each(|s| final_tokens.extend(s.clone().into_iter()));
        final_tokens
    }

}



fn write_starting_types() -> TokenStream {
    let tokens = quote! {
        pub struct AppContainer {
        }
        pub struct Component<T> {
            inner: Option<T>,
        }
        pub trait Container<T: Default> {
            fn get_create(&self) -> Component<T>;
        }
        impl <T> Component<T> {
            fn new(value: T) -> Self {
                Self {
                    inner: Some(value)
                }
            }
        }
    };
    tokens.into()
}



fn parse_module(mut found: Item) -> TokenStream {
    match &mut found {
        Item::Mod(ref mut struct_found) => {
            let mut container = ModuleContainer::default();
            parse_item_recursive(struct_found, &mut container);
            let container_tokens = container.to_token_stream();
            quote!(
                #found
                #container_tokens
            ).into()
        }
        _ => {
            return quote!(#found).into();
        }
    }
}



fn parse_item_recursive(item_found: &mut ItemMod, module_container: &mut ModuleContainer) {
    item_found.content.iter_mut()
        .flat_map(|mut c| c.1.iter_mut())
        .for_each(|i: &mut Item| parse_item(i, module_container));
}



fn get_trait(item_impl: &mut ItemImpl) -> Option<Path> {
    item_impl.trait_.clone()
        .and_then(|item_impl_found| {
            Some(item_impl_found.1)
        })
        .or_else(|| None)
}



fn parse_item(i: &mut Item, mut module_container: &mut ModuleContainer) {
    match i {
        Item::Const(_) => {}
        Item::Enum(_) => {}
        Item::ExternCrate(_) => {}
        Item::Fn(_) => {}
        Item::ForeignMod(_) => {}
        Item::Impl(impl_found) => {
            module_container.create_update_impl(impl_found);
        }
        Item::Macro(macro_created) => {
            // to add behavior to module macro,
            // have another macro impl Parse for a struct that
            // has a vec of Fn, and in the impl Parse
            // the behavior as a function that is added to the struct
            // to be called, and that function is passed as a closure
            // to the macro that creates the impl Parse - this will have to be
            // handled in the build.rs file - to relocate
            // macro_created.mac.parse_body()
        }
        Item::Macro2(_) => {}
        Item::Mod(ref mut module) => {
            println!("Found module with name {} !!!", module.ident.to_string().clone());
            parse_item_recursive(module, module_container);
        }
        Item::Static(_) => {}
        Item::Struct(ref mut item_struct) => {
            let f = TestFieldAdding {};
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
}



/**
Will be annotated with #[bean] and #[singleton], #[prototype] as provided factory functions.
 **/
struct ModulesFunctions {
    fn_found: ItemFn,
}

struct ModuleContainer {
    types: HashMap<String, DepImpl>,
    traits: HashMap<String, Trait>,
    fns: HashMap<String, ModulesFunctions>,
    profiles: Vec<Profile>,
}

impl ModuleContainer {

    fn to_token_stream(&self) -> proc_macro2::TokenStream {
        let mut token = quote! {};
        for token_type in &self.types {
            println!("Implementing container for {} if is not none.", token_type.1.id.clone());
            if token_type.1.struct_type.is_some() && token_type.1.traits_impl.iter().any(|p| p.to_token_stream().to_string().contains("Default")) {

                let struct_type =  token_type.1.struct_type.clone()
                    .unwrap().self_ty.deref().clone();

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
                            let this_component = Component::new::<#struct_type>();
                            this_component
                        }
                    }

                    impl Component<#struct_type> {
                        fn new() -> Self {
                            let mut inner = #struct_type::default();
                            #(
                                inner.#identifiers = AppContainer::get_create::<#field_types>();
                            )*
                            Component::new(Some(inner))
                        }
                    }
                };

                token.append_all(this_struct_impl);

            }
        }

        token
    }

    fn create_update_impl(&mut self, item_impl: &mut ItemImpl) {
        let id = item_impl.self_ty.to_token_stream().to_string().clone();
        &mut self.types.get_mut(&id)
            .map(|struct_impl: &mut DepImpl| {
                get_trait(item_impl).map(|path| struct_impl.traits_impl.push(path));
            })
            .or_else(|| {
                let impl_found = DepImpl {
                    struct_type: Some(item_impl.clone()),
                    struct_found: None,
                    traits_impl: get_trait(item_impl).map(|path| vec![path]).unwrap_or(vec![]),
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

    fn add_item_struct(&mut self, item_impl: &mut ItemStruct) {
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

    fn create_update_trait(&mut self, trait_found: &mut ItemTrait) {
        if !self.traits.contains_key(&trait_found.ident.to_string().clone()) {
            self.traits.insert(trait_found.ident.to_string().clone(), Trait::new(trait_found.clone()));
        } else {
            println!("Contained trait already!");
        }
    }

    fn set_deps(&mut self, item_impl: &mut ItemStruct) {
        match item_impl.fields.clone() {
            Fields::Named(fields_named) => {
                fields_named.named.iter().for_each(|field: &Field| {
                    field.clone().ident.map(|ident: Ident| {
                        println!("found field {}.", ident.to_string().clone());
                    });
                    println!("{} is the field type!", field.ty.to_token_stream().clone());
                    self.match_ty_recursive_add_container(item_impl, field.ty.clone(), false);
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
    fn match_ty_recursive_add_container(&mut self, item_impl: &mut ItemStruct, field: Type, is_ref: bool) {
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
                self.match_ty_recursive_add_container(item_impl, ref_type.clone().deref().clone(), true)
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

    fn match_ty_recursive_get_dependency(
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
                println!("Adding path: {}.", path.path.to_token_stream().to_string().clone());
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

    fn add_type(&mut self, item_impl: &mut ItemStruct, path: Path, is_ref: bool,
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
                .push(DepType { ident: Some(new_item_ident), id, is_ref: is_ref, type_found });
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

struct DepImpl {
    struct_type: Option<ItemImpl>,
    struct_found: Option<ItemStruct>,
    traits_impl: Vec<Path>,
    attr: Vec<Attribute>,
    // A reference to another DepImpl - the id is the Type.
    deps_map: Vec<DepType>,
    id: String,
    profile: Vec<Profile>,
    ident: Option<Ident>
}

struct Profile {
    profile: Vec<String>,
}

#[derive(Clone)]
struct DepType {
    id: String,
    is_ref: bool,
    type_found: Type,
    ident: Option<Ident>
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

struct Trait {
    trait_type: Option<ItemTrait>,
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


struct ApplicationContainer {
    modules: Vec<ModuleContainer>,
}

macro_rules! test_field_add {
    ($tt:tt) => {

    }
}

struct TestFieldAdding;

// A way to edit fields of structs - probably only possible to do through attributes..
impl TestFieldAdding {
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
