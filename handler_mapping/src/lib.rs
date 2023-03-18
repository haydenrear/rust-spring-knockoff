use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{Attribute, Block, FnArg, ImplItem, ImplItemMethod, Type};
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
            .flatten()
            .is_some()
    }

    fn create_controller_beans(bean: &Bean) -> Vec<ControllerBean> {
        bean.traits_impl.iter()
            .flat_map(|b| {
                b.item_impl.items.iter().flat_map(|i| {
                    match i {
                        ImplItem::Method(impl_item_method) => {
                            vec![(b.clone(), Self::create_request_matcher(&impl_item_method.attrs))]
                        }
                        _ => {
                            vec![]
                        }
                    }
                })
            })
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

    pub fn generate_token_stream(&self) -> TokenStream {
        let (to_match, split) = self.get_request_matcher_info();
        log_message!("{} are the number of to_match and {} are the number of split.", to_match.len(), split.len());

        if to_match.len() >= 1 {
            log_message!("{} are the inner number of to_match and {} are the number of split.", to_match.get(0).unwrap().len(), split.get(0).unwrap().len());
        }

        let args = self.controllers.iter()
            .flat_map(|c| c.arguments_resolved.iter().filter(|a| {
                a.request_body_arguments.len() != 0
            }).map(|r| {
                let args = r.request_body_arguments.get(0)
                    .unwrap();
                (Ident::new(args.inner.name.as_str(), Span::call_site()), args.request_serialize_type.clone(), args.output_type.clone().unwrap())
            }))
            .collect::<Vec<(Ident, Type, Type)>>();

        //TODO: These should be Vec<Vec<*>> because there will be tuples for multiple values...
        //  Then the tuple will be unwrapped accordingly.
        let arg_idents = args.iter().map(|i| i.0.clone()).collect::<Vec<Ident>>();
        let arg_types = args.iter().map(|i| i.1.clone()).collect::<Vec<Type>>();
        let arg_outputs = args.iter().map(|i| i.2.clone()).collect::<Vec<Type>>();

        log_message!("{} are the number of controllers.", self.controllers.len());
        let method_logic = self.controllers.iter()
            .map(|c| {
                log_message!("Here is the next block for controller bean: {}.", SynHelper::get_str(&c.method.block));
                &c.method.block
            })
            .collect::<Vec<&Block>>();

        let mut ts = quote! {

            use serde::{Serialize, Deserialize};
            use web_framework::web_framework::context::Context;
            use web_framework::web_framework::request_context::SessionContext;
            use web_framework_shared::request::WebRequest;
            use web_framework_shared::controller::{ContextData, Data, HandlerExecutionChain};
            use web_framework_shared::request::WebResponse;
            use web_framework::web_framework::context::UserRequestContext;
            use web_framework::web_framework::context::RequestContextData;
            use web_framework_shared::controller::{HandlerExecutor, RequestExecutor, HandlerMethod, HandlerExecutorStruct};
            use web_framework_shared::matcher::{AntPathRequestMatcher,AntStringRequestMatcher};

            pub struct Dispatcher {
                handler_mapping: AttributeHandlerMapping
            }

            impl RequestExecutor<WebRequest, WebResponse>
            for Dispatcher
            {
                fn do_request(&self, mut web_request: WebRequest) -> WebResponse {
                    self.handler_mapping.do_handle_request(web_request)
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
                impl HandlerExecutor<UserRequestContext<#arg_types>, RequestContextData<#arg_types, #arg_outputs>>
                for HandlerExecutorImpl<UserRequestContext<#arg_types>, RequestContextData<#arg_types, #arg_outputs>>
                {
                    fn execute_handler(
                        &self,
                        handler: &HandlerMethod<UserRequestContext<#arg_types>>,
                        ctx: &RequestContextData<#arg_types, #arg_outputs>,
                        response: &mut WebResponse,
                        request: &WebRequest
                    ) {
                        handler.request_ctx_data.clone().map(|#arg_idents| #method_logic);
                    }
                }
            )*

            impl AttributeHandlerMapping {
                pub fn new() -> Self {
                    #(
                        let mut interceptors = vec![];

                        let mut ant_string_request_matchers = vec![];
                        let mut request_matchers: Vec<AntPathRequestMatcher> = vec![];

                        #(
                            if #to_match.len() != 0 {
                                let path_matcher = AntStringRequestMatcher::new(#to_match.to_string(), #split.to_string());
                                ant_string_request_matchers.push(path_matcher);
                            }
                        )*

                        let ant_path_request_matchers: AntPathRequestMatcher = AntPathRequestMatcher::new_from_request_matcher(ant_string_request_matchers);
                        request_matchers.push(ant_path_request_matchers);

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

                fn do_handle_request(&self, mut request: WebRequest) -> Option<WebResponse> {
                    #(
                        if self.#arg_idents.matches(&request) {
                            return Some(self.#arg_idents.do_request(request));
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
        SynHelper::get_attr_from_vec(&attr, vec!["get_mapping", "post_mapping"])
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


