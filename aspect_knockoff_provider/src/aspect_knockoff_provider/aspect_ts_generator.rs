use std::collections::HashMap;
use proc_macro2::{Ident, TokenStream};
use module_macro_shared::bean::{BeanDefinition, BeanDefinitionType};
use module_macro_shared::profile_tree::ProfileTree;
use syn::{Block, Item, ItemFn, ItemImpl, Stmt, Type};
use codegen_utils::syn_helper::SynHelper;

use quote::{quote, TokenStreamExt, ToTokens};
use module_macro_shared::parse_container::MetadataItemId;
use crate::aspect_knockoff_provider::AspectInfo;

use knockoff_logging::*;
use lazy_static::lazy_static;
use std::sync::Mutex;
use codegen_utils::project_directory;
use collection_util::add_to_multi_value;
use crate::logger_lazy;
import_logger!("aspect_ts_generator.rs");

pub struct AspectGenerator {
    pub(crate) method_advice_aspects: Vec<(AspectInfo, BeanDefinition)>
}

impl AspectGenerator {

    /// This generates the aspect.
    pub fn generate_token_stream(&self) -> TokenStream {
        info!("Doing aspect generation.");
        let idents = self.method_advice_aspects.iter()
            .flat_map(|a| {
                let mut ts = TokenStream::default();
                a.1.traits_impl.iter().for_each(|d| {
                    info!("{:?} is itm impl", SynHelper::get_str(&d.item_impl));
                });
                a.1.ident.iter().map(move |i| {
                    Self::implement_original_fn(&mut ts, a);
                    (i.clone(), ts.clone())
                })
            })
            .collect::<Vec<(Ident, TokenStream)>>();

        let tys = self.method_advice_aspects.iter()
            .flat_map(|a| {
                let mut ts = TokenStream::default();
                a.1.traits_impl.iter().for_each(|d| {
                    info!("{:?} is itm impl", SynHelper::get_str(&d.item_impl));
                });
                a.1.struct_type.iter().map(move |i| {
                    Self::implement_original_fn(&mut ts, a);
                    (i.clone(), ts.clone())
                })
            })
            .collect::<Vec<(Type, TokenStream)>>();


        // merge these together if they are the same key
        let id = idents.iter().map(|i| i.0.clone().to_token_stream().to_string()).collect::<Vec<String>>();
        let ids = idents.iter().map(|i| i.0.clone()).collect::<Vec<Ident>>();
        let id_ts = idents.into_iter().map(|i| i.1).collect::<Vec<TokenStream>>();
        let ty = tys.iter().map(|i| i.0.clone().to_token_stream().to_string()).collect::<Vec<String>>();
        let tys_ = tys.iter().map(|i| i.0.clone()).collect::<Vec<Type>>();
        let ty_ts = tys.into_iter().map(|i| i.1).collect::<Vec<TokenStream>>();
        info!("Doing aspect generation with {}, {}.", ty.len(), ty_ts.len());
        info!("Found ids: {:?}", &id);
        info!("Found tys: {:?} ", &ty);

        ty_ts.iter().map(|t| SynHelper::get_str(t)).for_each(|i| {
            info!("{}", &i);
        });
        id_ts.iter().map(|t| SynHelper::get_str(t)).for_each(|i| {
            info!("{}", &i);
        });

        quote! {

            pub struct AspectGeneratorMutableModifier;

            impl MutableModuleModifier for AspectGeneratorMutableModifier {

                fn matches(item: &mut Item) -> bool {
                    match item {
                        Item::Impl(item_impl) => {
                            #(
                                if item_impl.self_ty.to_token_stream().to_string() == #ty.to_string()  {
                                    return true;
                                }
                            )*
                        }
                        Item::Fn(item_fn) => {
                            #(
                                if item_impl.sig.ident.to_token_stream().to_string() == #id.to_string()  {
                                    return true;
                                }
                            )*
                        }
                        // add functions
                        _ => {}
                    }
                    false
                }

                fn do_provide(item: &mut Item) -> Option<TokenStream> {
                    match item {
                        Item::Fn(item_fn) => {
                            #(
                                if item_impl.sig.ident.to_token_stream().to_string() == #id.to_string()  {
                                    return Some(quote! { #id_ts })
                                }
                            )*

                        }
                        Item::Impl(item_impl) => {
                            #(
                                if item_impl.self_ty.to_token_stream().to_string() == #ty.to_string()  {
                                    return Some(quote! { #ty_ts });
                                }
                            )*
                        }
                        _ => {}
                    }

                    None
                }

            }
        }

    }

}

impl AspectGenerator {

    pub fn new(profile_tree: &mut ProfileTree) -> Self {
        let method_advice_aspects = profile_tree.injectable_types.iter()
            .flat_map(|i_type| i_type.1)
            .flat_map(|bean_def| {
                info!("Looking for aspect info in aspect generator.");
                match bean_def {
                    BeanDefinitionType::Abstract { bean, dep_type } => {
                        vec![]
                    }
                    BeanDefinitionType::Concrete { bean } => {
                        let metadata_item = MetadataItemId::new(
                            bean.id.clone(),
                            "AspectInfo".to_string()
                        );
                        let aspects = profile_tree.provided_items.remove(&metadata_item)
                            .into_iter().flat_map(|removed| removed.into_iter())
                            .flat_map(|to_cast| {
                                info!("Found aspect info!");
                                AspectInfo::parse_values(&mut Some(to_cast))
                                    .map(|f| f.clone())
                                    .into_iter()
                            })
                            .collect::<Vec<AspectInfo>>();
                        aspects.into_iter()
                            .flat_map(|a| vec![(a.clone(), bean.clone())])
                            .collect::<Vec<(AspectInfo, BeanDefinition)>>()
                    }
                }
            }).collect::<Vec<(AspectInfo, BeanDefinition)>>();


        Self {
            method_advice_aspects
        }
    }

    /// When there is multiple advice, the last advice in the chain will call the original function logic,
    /// which means that the original function logic will be implemented in a trait with the name of
    /// the proceed statement in the last advice of the chain.
    ///
    pub(crate) fn implement_proceed_original_fn_logic(
        mut ts: &mut TokenStream, a: &(AspectInfo, BeanDefinition),
        block: &Block, arg_idents: &Vec<&Ident>, arg_types: &Vec<&Type>
    ) {
        let mut proceed_suffix = "".to_string();

        if a.0.advice_chain.len() == 0 {
            proceed_suffix = Self::get_suffix(&a.0.method_advice_aspect.proceed_statement);
        } else {
            proceed_suffix = Self::get_suffix(&a.0.advice_chain.last().as_ref().unwrap().proceed_statement);
            log_message!("{} is the last proceed statement suffix.", proceed_suffix);
        }

        let method_ident = &a.0.method.as_ref().unwrap().sig.ident;

        Self::implement(ts, a, &arg_idents, &arg_types, &mut proceed_suffix, &mut vec![block.to_token_stream().clone()], method_ident);
    }

    /// The proceed statement in the advice is calling a function on a trait that contains
    /// the logic for the next advice. Therefore, when implementing that trait for the
    /// next advice, you must use the suffix of the proceed statement in the previous link
    /// in the advice chain.
    /// ** The first link in the chain, which is the original function call,
    /// and the last advice in the chain, which is the original function logic, do not need
    /// to be implemented. **
    pub fn implement_chain(
        mut ts: &mut TokenStream, a: &(AspectInfo, BeanDefinition),
        arg_idents: &Vec<&Ident>, arg_types: &Vec<&Type>,
    ) {
        let advice_chain_len = a.0.advice_chain.len();
        if advice_chain_len > 0 {
            let mut proceed_suffix = "".to_string();
            for advice_index in 0..a.0.advice_chain.len() {
                a.0.advice_chain.get(advice_index).map(|next_chain| {
                    let mut block_items = vec![];

                    next_chain.before_advice
                        .as_ref()
                        .map(|b| block_items.push(b.to_token_stream().clone()));

                    next_chain.proceed_statement
                        .as_ref()
                        .map(|b| block_items.push(b.to_token_stream().clone()));

                    next_chain.after_advice
                        .as_ref()
                        .map(|b| block_items.push(b.to_token_stream().clone()));

                    if advice_index > 0 {
                        proceed_suffix = a.0.advice_chain[advice_index - 1].proceed_statement
                            .as_ref()
                            .map(|s| SynHelper::get_proceed(s.to_token_stream().to_string().clone()))
                            .or(Some("".to_string()))
                            .unwrap();

                        log_message!("Implementing next: {}.", &proceed_suffix);

                    } else {
                        proceed_suffix = SynHelper::get_proceed(a.0.method_advice_aspect.proceed_statement.as_ref().unwrap().to_token_stream().to_string().clone());
                        log_message!("Implementing initial: {}.", &proceed_suffix);
                    }

                    if proceed_suffix.len() != 0 {
                        let method_ident = &a.0.method.as_ref().unwrap().sig.ident;
                        Self::implement(ts, a, &arg_idents, &arg_types, &mut proceed_suffix, &mut block_items, method_ident);
                    }

                });
            }
        }
    }

    pub(crate) fn implement(
        mut ts: &mut TokenStream,
        a: &(AspectInfo, BeanDefinition),
        arg_idents: &Vec<&Ident>,
        arg_types: &Vec<&Type>,
        proceed_suffix: &mut String,
        block_items: &mut Vec<TokenStream>,
        method_ident: &Ident
    ) {

        a.1.struct_type.as_ref().map(|struct_type| {
            a.0.return_type.as_ref().map(|return_type| {
                if a.0.mutable {
                    ts.append_all(
                        Self::proceed_with_return_type_mutable(
                            &proceed_suffix, method_ident,
                            &block_items, &arg_idents, &arg_types,
                            struct_type, return_type,
                        ));
                } else {
                    ts.append_all(
                        Self::proceed_with_return_type(
                            &proceed_suffix, method_ident,
                            &block_items, &arg_idents, &arg_types,
                            struct_type, return_type,
                        ));
                }
            }).or_else(|| {
                if a.0.mutable {
                    ts.append_all(
                        Self::proceed_no_return_type_mutable(
                            &proceed_suffix, method_ident,
                            &block_items, &arg_idents, &arg_types,
                            struct_type,
                        ));
                } else {
                    ts.append_all(
                        Self::proceed_no_return_type(
                            &proceed_suffix, method_ident,
                            &block_items, &arg_idents, &arg_types,
                            struct_type,
                        ));
                }
                None
            });
        });
    }

    pub(crate) fn implement_original_fn(mut ts: &mut TokenStream, a: &(AspectInfo, BeanDefinition)) {
        let block = a.0.original_fn_logic.as_ref().unwrap();
        let arg_idents = a.0.args.iter().map(|a| &a.0).collect::<Vec<&Ident>>();
        let arg_types = a.0.args.iter().map(|a| &a.1).collect::<Vec<&Type>>();
        Self::implement_chain(ts, a, &arg_idents, &arg_types);
        Self::implement_proceed_original_fn_logic(&mut ts, a, block, &arg_idents, &arg_types);
    }

    fn proceed_with_return_type<T: ToTokens>(
        suffix: &String, method_name: &Ident, block: &Vec<T>, arg_idents: &Vec<&Ident>,
        arg_types: &Vec<&Type>, struct_type: &Type, return_type: &Type
    ) -> TokenStream {
        log_message!("Implementing aspect with suffix {} and method name {}.", suffix.as_str(), method_name.to_string().as_str());
        let aspect_tokens =
            quote! {

                paste! {

                    pub trait [<#suffix  #struct_type>] {
                        fn [<proceed #suffix>](&self, #(#arg_idents: #arg_types),*) -> #return_type;
                    }

                    impl [<#suffix #struct_type>] for #struct_type {
                        fn [<proceed #suffix>](&self, #(#arg_idents: #arg_types),*) -> #return_type {
                            #(#block)*
                        }
                    }
                }
            };

        aspect_tokens
    }

    fn proceed_with_return_type_mutable<T: ToTokens>(
        suffix: &String, method_name: &Ident, block: &Vec<T>, arg_idents: &Vec<&Ident>,
        arg_types: &Vec<&Type>, struct_type: &Type, return_type: &Type
    ) -> TokenStream {
        log_message!("Implementing aspect with suffix {} and method name {}.", suffix.as_str(), method_name.to_string().as_str());
        let aspect_tokens =
            quote! {

                paste! {

                    pub trait [<#suffix  #struct_type>] {
                        fn [<proceed #suffix>](&mut self, #(#arg_idents: #arg_types),*) -> #return_type;
                    }

                    impl [<#suffix #struct_type>] for #struct_type {
                        fn [<proceed #suffix>](&mut self, #(#arg_idents: #arg_types),*) -> #return_type {
                            #(#block)*
                        }
                    }
                }
            };

        aspect_tokens
    }

    fn proceed_no_return_type<T: ToTokens>(suffix: &String, method_name: &Ident, block: &Vec<T>, arg_idents: &Vec<&Ident>, arg_types: &Vec<&Type>, struct_type: &Type) -> TokenStream {
        log_message!("Implementing aspect with suffix {} and method name {}.", suffix.as_str(), method_name.to_string().as_str());
        let aspect_tokens =
            quote! {

                paste! {

                    pub trait [<#method_name  #struct_type>] {
                        fn [<proceed #suffix](&self, #(#arg_idents: #arg_types),*);
                    }

                    impl [<#method_name #struct_type>] for #struct_type {
                        fn [<proceed #suffix](&self, #(#arg_idents: #arg_types),*) {
                            #(#block)*
                        }
                    }
                }
            };
        aspect_tokens
    }

    fn proceed_no_return_type_mutable<T: ToTokens>(suffix: &String, method_name: &Ident, block: &Vec<T>, arg_idents: &Vec<&Ident>, arg_types: &Vec<&Type>, struct_type: &Type) -> TokenStream {
        log_message!("Implementing aspect with suffix {} and method name {}.", suffix.as_str(), method_name.to_string().as_str());
        let aspect_tokens =
            quote! {

                paste! {

                    pub trait [<#method_name  #struct_type>] {
                        fn [<proceed #suffix](&mut self, #(#arg_idents: #arg_types),*);
                    }

                    impl [<#method_name #struct_type>] for #struct_type {
                        fn [<proceed #suffix](&mut self, #(#arg_idents: #arg_types),*) {
                            #(#block)*
                        }
                    }
                }
            };
        aspect_tokens
    }

    fn get_suffix(method: &Option<Stmt>) -> String {
        method
            .as_ref()
            .map(|m| {
                log_message!("Checking for proceed: {}", SynHelper::get_str(m));
                SynHelper::get_proceed(m.to_token_stream().to_string().clone())
            })
            .or(Some("".to_string()))
            .map(|aspect_suffix| {
                log_message!("{} is the proceed part to be used to create trait.", &aspect_suffix);
                aspect_suffix
            })
            .unwrap()
    }

}
