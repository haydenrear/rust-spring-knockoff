use proc_macro2::{Ident, TokenStream};
use quote::{quote, TokenStreamExt};
use syn::{Block, Type};
use module_macro_codegen::aspect::{AspectParser, MethodAdviceAspectCodegen};
use crate::module_macro_lib::knockoff_context_builder::token_stream_generator::TokenStreamGenerator;
use crate::module_macro_lib::module_tree::{AspectInfo, Bean, BeanDefinitionType};
use crate::module_macro_lib::profile_tree::ProfileTree;

pub struct AspectGenerator {
    method_advice_aspects: Vec<(AspectInfo, Bean)>
}

impl TokenStreamGenerator for AspectGenerator {
    fn generate_token_stream(&self) -> TokenStream {
        let mut ts = TokenStream::default();
        self.method_advice_aspects.iter()
            .for_each(|a| Self::implement_aspect_ty(&mut ts, a));
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
                            bean.aspect_info.clone().map(|a| vec![(a,bean.clone())])
                                .or(Some(vec![]))
                                .unwrap()
                        }
                    }
                })
            }).collect::<Vec<(AspectInfo, Bean)>>();


        Self {
            method_advice_aspects
        }
    }

    fn proceed_with_return_type(block: &Block, arg_idents: &Vec<&Ident>, arg_types: &Vec<&Type>, struct_type: &Type, return_type: &Type) -> TokenStream {
        let aspect_tokens = quote! {

            impl #struct_type {
                pub fn proceed(&self, #(#arg_idents: #arg_types),*) -> #return_type {
                    #block
                }
            }

        };
        aspect_tokens
    }

    fn proceed_no_return_type(block: &Block, arg_idents: &Vec<&Ident>, arg_types: &Vec<&Type>, struct_type: &Type) -> TokenStream {
        let aspect_tokens = quote! {

            impl #struct_type {
                pub fn proceed(&self, #(#arg_idents: #arg_types),*) {
                    #block
                }
            }

        };
        aspect_tokens
    }

    fn implement_proceed_for_aspect_type(mut ts: &mut TokenStream, a: &(AspectInfo, Bean), block: &Block, arg_idents: &Vec<&Ident>, arg_types: &Vec<&Type>) {
        a.1.struct_type.as_ref().map(|struct_type| {
            a.0.return_type.as_ref().map(|return_type| {
                let aspect_tokens = Self::proceed_with_return_type(
                    &block, &arg_idents, &arg_types,
                    struct_type, return_type
                );
                ts.append_all(aspect_tokens);
            }).or_else(|| {
                let aspect_tokens = Self::proceed_no_return_type(
                    block, arg_idents,
                    arg_types, struct_type
                );
                ts.append_all(aspect_tokens);
                None
            });
        });
    }

    fn implement_aspect_ty(mut ts: &mut TokenStream, a: &(AspectInfo, Bean)) {
        let block = a.0.block.as_ref().unwrap();
        let arg_idents = a.0.args.iter().map(|a| &a.0).collect::<Vec<&Ident>>();
        let arg_types = a.0.args.iter().map(|a| &a.1).collect::<Vec<&Type>>();
        Self::implement_proceed_for_aspect_type(&mut ts, a, block, &arg_idents, &arg_types);
    }
}