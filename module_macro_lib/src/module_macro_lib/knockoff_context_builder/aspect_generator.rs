use proc_macro2::TokenStream;
use module_macro_codegen::aspect::{AspectParser, MethodAdviceAspectCodegen};
use crate::module_macro_lib::knockoff_context_builder::token_stream_generator::TokenStreamGenerator;
use crate::module_macro_lib::module_tree::{Bean, BeanDefinitionType};
use crate::module_macro_lib::profile_tree::ProfileTree;

pub struct AspectGenerator {
    method_advice_aspects: Vec<(MethodAdviceAspectCodegen, Bean)>
}

impl TokenStreamGenerator for AspectGenerator {
    fn generate_token_stream(&self) -> TokenStream {
        TokenStream::default()
    }
}

impl AspectGenerator {
    pub fn new(profile_tree: ProfileTree) -> Self {
        let aspects = AspectParser::parse_aspects();
        let method_advice_aspects = profile_tree.injectable_types.iter()
            .flat_map(|i_type| {
                i_type.1.iter().flat_map(|bean_def| {
                    match bean_def {
                        BeanDefinitionType::Abstract { bean, dep_type} => {
                            vec![]
                        }
                        BeanDefinitionType::Concrete { bean } => {
                            aspects.aspects.iter()
                                .flat_map(|a| &a.method_advice_aspects)
                                .flat_map(|a| {
                                    if MethodAdviceAspectCodegen::aspect_matches(&bean.path_depth, &a.pointcut, &bean.id) {
                                        return vec![(a.clone(), bean.clone())]
                                    }
                                    vec![]
                                }).collect::<Vec<(MethodAdviceAspectCodegen, Bean)>>()
                        }
                    }
                })
            }).collect::<Vec<(MethodAdviceAspectCodegen, Bean)>>();


        Self {
            method_advice_aspects
        }
    }
}