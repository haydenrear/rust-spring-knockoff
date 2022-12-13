extern crate proc_macro;

use delegator_macro_rules::types;
use lazy_static::lazy_static;
use proc_macro::{Span, TokenStream};
use std::any::{Any, TypeId};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, LinkedList};
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use syn::{parse_macro_input, DeriveInput, Data, Fields, Field, Item, ItemMod, ItemStruct, FieldsNamed, FieldsUnnamed, ItemImpl, ImplItem, ImplItemMethod, parse_quote, parse, Type};
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
            parse_item_recursive(struct_found);
            return quote!(#found).into();
        }
        _ => {
            return quote!(#found).into();
        }
    }
}

fn parse_item_recursive(item_found: &mut ItemMod) {
    item_found.content.iter_mut()
        .flat_map(|mut c| c.1.iter_mut())
        .for_each(|i: &mut Item|  parse_item(i));
}

fn parse_item(i: &mut Item) {
    match i {
        Item::Const(_) => {}
        Item::Enum(_) => {}
        Item::ExternCrate(_) => {}
        Item::Fn(_) => {}
        Item::ForeignMod(_) => {}
        Item::Impl(impl_found) => {
            // if let Some((Some(token), path, token_for)) = &impl_found.trait_ {
            //     let trait_implemented = path.segments[0].ident.clone();
            //     reify!()
            // }
            // let created = reify!(impl_found.self_ty.into_token_stream());
            // println!("Created!");
        }
        Item::Macro(_) => {}
        Item::Macro2(_) => {}
        Item::Mod(ref mut module) => {
            println!("Found module with name {} !!!", module.ident.to_string().clone());
            parse_item_recursive(module);
        }
        Item::Static(_) => {}
        Item::Struct(ref mut item_struct) => {
            let f = ImplRunningPostProcessor{};
            f.process(item_struct);
            println!("Found struct with name {} !!!", item_struct.ident.to_string().clone());
        }
        Item::Trait(trait_created) => {
            println!("Trait created: {}", trait_created.ident.clone().to_string())
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