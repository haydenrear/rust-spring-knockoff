use proc_macro2::TokenStream;
use syn::{Attribute, ImplItem, ImplItemMethod};
use codegen_utils::syn_helper::SynHelper;
use module_macro_shared::bean::BeanDefinitionType;
use module_macro_shared::profile_tree::ProfileTree;
use token_stream_provider::TokenStreamProvider;
use web_framework_shared::matcher::AntPathRequestMatcher;
use web_framework_shared::request::WebRequest;
use crate::request::WebRequest;

pub struct HandlerMappingBuilder {
    controllers: Vec<ControllerBean>,
}

impl TokenStreamProvider for HandlerMappingBuilder {

    fn generate_token_stream(&self) -> TokenStream {
        TokenStream::default()
    }

    fn new(items: &ProfileTree) -> Self {
        let controller_beans = items.injectable_types.iter()
            .flat_map(|b| b.1.iter())
            .flat_map(|b| match b {
                BeanDefinitionType::Abstract { .. } => {
                    return vec![];
                }
                BeanDefinitionType::Concrete { bean } => {
                    let ant_matcher = bean.struct_found
                        .map(|s| Self::create_request_matcher(&s.attrs))
                        .flatten();
                    if ant_matcher.is_some() {
                        return bean.traits_impl.iter()
                            .flat_map(|t| t.item_impl.items.iter())
                            .flat_map(|i| match i {
                                ImplItem::Method(impl_item_method) => {
                                    vec![ControllerBean { method: impl_item_method.clone(), ant_path_request_matcher: ant_matcher.unwrap() }]
                                }
                                _ => {
                                    vec![]
                                }
                            }).collect::<Vec<ControllerBean>>();
                    }
                    vec![]
                }
            })
            .collect::<Vec<ControllerBean>>();
        Self {
            controllers: controller_beans
        }

    }
}

impl HandlerMappingBuilder {

    pub(crate) fn create_request_matcher(attr: &Vec<Attribute>) -> Option<AntPathRequestMatcher> {
        None
    }
}

struct ControllerBean {
    method: ImplItemMethod,
    ant_path_request_matcher: AntPathRequestMatcher,
}

