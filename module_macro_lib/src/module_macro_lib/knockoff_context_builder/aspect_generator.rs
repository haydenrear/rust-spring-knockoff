use proc_macro2::{Ident, TokenStream};
use quote::{quote, TokenStreamExt, ToTokens};
use syn::{Block, Expr, ImplItemMethod, Stmt, Type};
use codegen_utils::syn_helper::SynHelper;
use knockoff_logging::{initialize_log, use_logging};
use module_macro_codegen::aspect::{AspectParser, MethodAdviceAspectCodegen};
use crate::module_macro_lib::knockoff_context_builder::token_stream_generator::TokenStreamGenerator;
use crate::module_macro_lib::module_tree::{AspectInfo, Bean, BeanDefinitionType};

use_logging!();
initialize_log!();

use crate::module_macro_lib::logging::executor;
use crate::module_macro_lib::logging::StandardLoggingFacade;
use crate::module_macro_lib::profile_tree::ProfileTree;


pub struct AspectGenerator {
    method_advice_aspects: Vec<(AspectInfo, Bean)>
}

impl TokenStreamGenerator for AspectGenerator {
    fn generate_token_stream(&self) -> TokenStream {
        let mut ts = TokenStream::default();
        self.method_advice_aspects.iter()
            .for_each(|a| {
                Self::implement_original_fn(&mut ts, a);
            });
        ts
    }
}

impl AspectGenerator {

    pub fn new(profile_tree: &ProfileTree) -> Self {
        let method_advice_aspects = profile_tree.injectable_types.iter()
            .flat_map(|i_type| {
                i_type.1.iter().flat_map(|bean_def| {
                    match bean_def {
                        BeanDefinitionType::Abstract { bean, dep_type} => {
                            vec![]
                        }
                        BeanDefinitionType::Concrete { bean } => {
                            bean.aspect_info.iter()
                                .flat_map(|a| vec![(a.clone(),bean.clone())])
                                .collect::<Vec<(AspectInfo, Bean)>>()
                        }
                    }
                })
            }).collect::<Vec<(AspectInfo, Bean)>>();


        Self {
            method_advice_aspects
        }
    }

    /// The function that was originally being called is placed in a trait, and then if there are
    /// additional functions to be called, (multiple advice) then in that one that is called a proceed
    /// statement is placed there.
    fn implement_proceed_original_fn_logic(
        mut ts: &mut TokenStream, a: &(AspectInfo, Bean),
        block: &Block, arg_idents: &Vec<&Ident>, arg_types: &Vec<&Type>
    ) {
        let mut proceed_suffix = "".to_string();

        if a.0.advice_chain.len() == 0 {
            proceed_suffix = Self::get_suffix(&a.0.method_advice_aspect.proceed_statement);
        } else {
            proceed_suffix = Self::get_suffix(&a.0.advice_chain.last().as_ref().unwrap().proceed_statement)
        }

        let method_ident = &a.0.method.as_ref().unwrap().sig.ident;

        Self::implement(ts, a, &arg_idents, &arg_types, &mut proceed_suffix, &mut vec![block.to_token_stream().clone()], method_ident);
    }

    pub fn implement_chain(
        mut ts: &mut TokenStream, a: &(AspectInfo, Bean), block: &Block,
        arg_idents: &Vec<&Ident>, arg_types: &Vec<&Type>,
    ) {
        let mut proceed_suffix = "".to_string();
        let mut counter = 0;
        let advice_chain_len = a.0.advice_chain.len();
        if advice_chain_len > 0 {
            for next_chain in a.0.advice_chain.iter() {
                let mut block_items = vec![];

                next_chain.before_advice.as_ref().map(|b| block_items.push(b.to_token_stream().clone()));
                next_chain.proceed_statement.as_ref().map(|b| block_items.push(b.to_token_stream().clone()));
                next_chain.after_advice.as_ref().map(|b| block_items.push(b.to_token_stream().clone()));

                if proceed_suffix == "" {
                    proceed_suffix = SynHelper::get_proceed(a.0.method_advice_aspect.proceed_statement.as_ref().unwrap().to_token_stream().to_string().clone());
                } else if counter < advice_chain_len - 1 {
                    proceed_suffix = a.0.advice_chain[counter + 1].proceed_statement.as_ref()
                        .map(|s| SynHelper::get_proceed(s.to_token_stream().to_string().clone()))
                        .or(Some("".to_string()))
                        .unwrap();
                }

                if proceed_suffix.len() == 0 {
                    break;
                }

                let method_ident = &a.0.method.as_ref().unwrap().sig.ident;

                Self::implement(ts, a, &arg_idents, &arg_types, &mut proceed_suffix, &mut block_items, method_ident);
                counter += 1;
            }
        }
    }

    fn implement(mut ts: &mut TokenStream, a: &(AspectInfo, Bean), arg_idents: &&Vec<&Ident>, arg_types: &&Vec<&Type>, proceed_suffix: &mut String, block_items: &mut Vec<TokenStream>, method_ident: &Ident) {
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

    fn implement_original_fn(mut ts: &mut TokenStream, a: &(AspectInfo, Bean)) {
        let block = a.0.original_fn_logic.as_ref().unwrap();
        let arg_idents = a.0.args.iter().map(|a| &a.0).collect::<Vec<&Ident>>();
        let arg_types = a.0.args.iter().map(|a| &a.1).collect::<Vec<&Type>>();
        Self::implement_chain(ts, a, block, &arg_idents, &arg_types);
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