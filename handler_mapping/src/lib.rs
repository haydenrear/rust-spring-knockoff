use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{Attribute, FnArg, ImplItem, ImplItemMethod, Type};
use codegen_utils::project_directory;
use codegen_utils::syn_helper::SynHelper;
use knockoff_logging::log_message;
use module_macro_shared::bean::{Bean, BeanDefinitionType};
use module_macro_shared::profile_tree::ProfileTree;
use web_framework_shared::matcher::{AntPathRequestMatcher, AntStringRequestMatcher, Matcher};

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
        let (to_match, split) = self.get_request_matcher_info();
        let args = self.controllers.iter().flat_map(|c| c.arguments_resolved.iter().filter(|a| {
            a.request_body_arguments.len() != 0
        }).map(|r| {
            let args = r.request_body_arguments.get(0).unwrap();
            (Ident::new(args.inner.name.as_str(), Span::call_site()), args.request_serialize_type.clone(), args.output_type.clone().unwrap())
        }))
            .collect::<Vec<(Ident, Type, Type)>>();

        let arg_idents = args.iter().map(|i| i.0.clone()).collect::<Vec<Ident>>();
        let arg_types = args.iter().map(|i| i.1.clone()).collect::<Vec<Type>>();
        let arg_outputs = args.iter().map(|i| i.2.clone()).collect::<Vec<Type>>();

        let mut ts = quote! {

            use serde::{Serialize, Deserialize};
            use web_framework::web_framework::context::Context;
            use web_framework::web_framework::request_context::RequestContext;
            use web_framework_shared::request::WebRequest;
            use web_framework_shared::controller::{ContextData, Data, HandlerExecutionChain};
            use web_framework_shared::request::WebResponse;
            use web_framework::web_framework::context::UserRequestContext;
            use web_framework::web_framework::context::RequestContextData;
            use web_framework_shared::controller::{HandlerExecutor, RequestExecutor, HandlerMethod, HandlerExecutorStruct};

            pub struct Dispatcher {
                handler_mapping: AttributeHandlerMapping
            }

            impl RequestExecutor<WebRequest, WebResponse>
            for Dispatcher
            {
                fn do_request(&self, mut web_request: WebRequest) -> WebResponse {
                    self.handler_mapping.get_handler(&web_request)
                        .map(|handler| {
                            handler.do_request(web_request.clone())
                        })
                        .or(Some(WebResponse::default()))
                        .unwrap()
                }
            }

            pub struct AttributeHandlerMapping {
                #(#arg_idents: Arc<HandlerExecutionChain<UserRequestContext<#arg_types>, RequestContextData<#arg_types, #arg_outputs>>>,)*
            }

            pub struct HandlerExecutorImpl <D: Data + Send + Sync, C: ContextData + Send + Sync> {
                phantom_d: PhantomData<D>,
                phantom_c: PhantomData<C>
            }

            impl <D: Data + Send + Sync, C: ContextData + Send + Sync> Default for HandlerExecutorImpl<D, C> {
                fn default() -> Self {
                    Self {
                        phantom_d: PhantomData::default(),
                        phantom_c: PhantomData::default()
                    }
                }
            }

            #(
                impl HandlerExecutor<UserRequestContext<#arg_types>, RequestContextData<#arg_types, #arg_outputs>> for HandlerExecutorImpl<UserRequestContext<#arg_types>, RequestContextData<#arg_types, #arg_outputs>> {
                    fn execute_handler(&self, handler: &HandlerMethod<UserRequestContext<#arg_types>>, ctx: &RequestContextData<#arg_types, #arg_outputs>, response: &mut WebResponse, request: &WebRequest) {
                        todo!()
                    }
                }
            )*

            impl AttributeHandlerMapping {
                pub fn new() -> Self {
                    #(
                        let mut interceptors = vec![];
                        let mut request_matchers = vec![];

                        #(
                            let path_matcher = AntPathRequestMatcher::new(#to_match, #split);
                            request_matchers.push(path_matcher);
                        )*

                        let context_item = Context::<#arg_types, #arg_outputs>::new();
                        let context: Arc<RequestContextData<#arg_types, #arg_outputs>> = Arc::new(
                            RequestContextData {
                                request_context_data: context_item
                            }
                        );

                        let handler_executor: Arc<HandlerExecutorImpl<UserRequestContext<#arg_types>, RequestContextData<#arg_types, #arg_outputs>>>
                            = Arc::new(HandlerExecutorImpl::default());

                        let handler_executor = handler_executor as Arc<dyn HandlerExecutor<
                                            UserRequestContext<#arg_types>,
                                            RequestContextData<#arg_types, #arg_outputs>
                        >>;

                        let handler_executor: Arc<HandlerExecutorStruct<
                            dyn HandlerExecutor<UserRequestContext<#arg_types>, RequestContextData<#arg_types, #arg_outputs>>,
                            UserRequestContext<#arg_types>,
                            RequestContextData<#arg_types, #arg_outputs>
                        >> = Arc::new(HandlerExecutorStruct {
                            handler_executor,
                            phantom_data_t: PhantomData::default(),
                            phantom_data_ctx: PhantomData::default()
                        });

                        let #arg_idents = Arc::new(HandlerExecutionChain {
                            interceptors: Arc::new(interceptors),
                            request_matchers,
                            handler_executor,
                            context
                        });
                    )*

                    Self {
                        #(#arg_idents,)*
                    }
                }
            }

            pub trait GetHandlerMapping {
                fn get_handler(&self, request: &WebRequest) -> Option<Arc<HandlerExecutionChain<dyn Data, dyn ContextData>>>;
            }

            impl GetHandlerMapping for AttributeHandlerMapping {
                fn get_handler(&self, request: &WebRequest) -> Option<Arc<HandlerExecutionChain<dyn Data, dyn ContextData>>>{
                    #(
                        if self.#arg_idents.matches(request) {
                            let any_val = self.#arg_idents.clone() as Arc<dyn Any + Send + Sync>;
                            return any_val.downcast::<HandlerExecutionChain<dyn Data, dyn ContextData>>()
                                .ok();
                        }
                    )*
                    None
                }
            }

        };
        ts.into()
    }

    fn get_request_matcher_info(&self) -> (Vec<Vec<String>>, Vec<Vec<String>>) {
        let to_match = self.controllers.iter()
            .map(|m| m.ant_path_request_matcher.iter()
                .flat_map(|r| r.request_matchers.iter()
                    .map(|r| r.to_match.clone())
                ).collect::<Vec<String>>()
            ).collect::<Vec<Vec<String>>>();
        let split = self.controllers.iter()
            .map(|m| m.ant_path_request_matcher.iter()
                .flat_map(|r| r.request_matchers.iter()
                    .map(|r| r.splitter.clone())
                ).collect::<Vec<String>>()
            ).collect::<Vec<Vec<String>>>();
        (to_match, split)
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


