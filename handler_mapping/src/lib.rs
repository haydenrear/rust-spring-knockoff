use proc_macro2::TokenStream;
use quote::quote;
use syn::{Attribute, FnArg, ImplItem, ImplItemMethod, Type};
use codegen_utils::project_directory;
use codegen_utils::syn_helper::SynHelper;
use knockoff_logging::log_message;
use module_macro_shared::bean::{Bean, BeanDefinitionType};
use module_macro_shared::profile_tree::ProfileTree;
use web_framework_shared::matcher::{AntPathRequestMatcher, AntStringRequestMatcher};

use module_macro_shared::logging::StandardLoggingFacade;
use module_macro_shared::logging::executor;
use module_macro_shared::logging::logger;

use knockoff_logging::knockoff_logging::logging_facade::LoggingFacade;
use knockoff_logging::knockoff_logging::log_level::LogLevel;
use executors::common::Executor;
use knockoff_logging::knockoff_logging::logger::Logger;
use module_macro_shared::dependency::AutowireType;
use web_framework_shared::argument_resolver::{ArgumentResolver, ResolveArguments};

pub struct HandlerMappingBuilder {
    controllers: Vec<ControllerBean>,
}

impl HandlerMappingBuilder {

    pub fn new(items: &ProfileTree) -> Self {
        let controller_beans = items.injectable_types.iter()
            .flat_map(|b| b.1.iter())
            .flat_map(|b| match b {
                BeanDefinitionType::Abstract { .. } => {
                    vec![]
                }
                BeanDefinitionType::Concrete { bean } => {
                    vec![bean]
                }
            })
            .filter(|b| Self::filter_controller_beans(b))
            .flat_map(|bean| Self::create_controller_beans(bean))
            .collect::<Vec<ControllerBean>>();

        Self {
            controllers: controller_beans
        }

    }

    fn filter_controller_beans(b: &&Bean) -> bool {
        b.struct_found.as_ref()
            .map(|s| SynHelper::get_attr_from_vec(
                &s.attrs,
                vec!["controller", "rest_controller"])
            )
            .is_some()
    }

    fn create_controller_beans(bean: &Bean) -> Vec<ControllerBean> {
        bean.traits_impl.iter()
            .map(|autowire_type| (
                autowire_type.clone(),
                Self::create_request_matcher(&autowire_type.item_impl.attrs)
            ))
            .flat_map(|autowire| autowire.0.item_impl.items
                .iter()
                .map(|i| (autowire.0.clone(), autowire.1.clone(), i.clone()))
                .collect::<Vec<(AutowireType, Vec<AntPathRequestMatcher>, ImplItem)>>()
            )
            .flat_map(|i| match i.2 {
                ImplItem::Method(impl_item_method) => {
                    vec![ControllerBean {
                        method: impl_item_method.clone(),
                        ant_path_request_matcher: i.1.clone(),
                        arguments_resolved: ArgumentResolver::resolve_argument_methods(&impl_item_method)
                    }]
                }
                _ => {
                    vec![]
                }
            }).collect::<Vec<ControllerBean>>()
    }

    // fn get_inputs(impl_item_method: &ImplItemMethod) {
    //     let input_type = impl_item_method.sig.inputs.iter()
    //         .map(|fn_arg| {
    //             match fn_arg {}
    //         })
    // }

    pub fn generate_token_stream(&self) -> TokenStream {
        let mut ts = quote! {
            pub struct RequestContextData<Request, Response>
            where
                Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
                Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static
            {
                request_context_data: Context<Request, Response>
            }

            impl <Request, Response> ContextData for RequestContextData<Request, Response>
            where
                Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
                Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static {}

            pub struct UserRequestContext<Request>
            where
                Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static
            {
                request_context: RequestContext,
                request: Request
            }

            impl <Request> Data for UserRequestContext<Request>
            where
                Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static{}

            pub struct AttributeHandlerMapping {
                //TODO: field for each handler mapping... OR downcast to get.
            }

            pub trait GetHandlerMapping<T: HandlerExecutionChain<D, C>, D: Data, C: ContextData> {
                fn get_handler(&self, request: &WebRequest) -> HandlerExecutionChain<dyn Data, dyn ContextData>;
            }

            pub trait HandlerMapping {
            }

        };
        TokenStream::default()
    }
}

impl HandlerMappingBuilder {

    pub(crate) fn create_request_matcher(attr: &Vec<Attribute>) -> Vec<AntPathRequestMatcher> {
        SynHelper::get_attr_from_vec(&attr, vec!["get_mapping"])
            .or(SynHelper::get_attr_from_vec(&attr, vec!["post_mapping"]))
            .map(|attr| {
                log_message!("{} is the controller mapping for creating HandlerMapping.", &attr);
                attr.split(",")
                    .map(|s| s.replace(" ", ""))
                    .collect::<Vec<String>>()
            })
            .or(Some(vec![]))
            .unwrap()
            .iter()
            .map(|s| AntPathRequestMatcher::new(s.as_str(), "/"))
            .collect::<Vec<AntPathRequestMatcher>>()
    }
}

struct ControllerBean {
    method: ImplItemMethod,
    ant_path_request_matcher: Vec<AntPathRequestMatcher>,
    arguments_resolved: Vec<ArgumentResolver>
}


