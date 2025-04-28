use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{Attribute, Block, FnArg, ImplItem, ImplItemMethod, Stmt, Type};
use codegen_utils::syn_helper::SynHelper;
use knockoff_logging::log_message;
use module_macro_shared::bean::{BeanDefinition, BeanDefinitionType};
use module_macro_shared::profile_tree::ProfileTree;
use web_framework_shared::matcher::{AntPathRequestMatcher, AntStringRequestMatcher, Matcher};

use module_macro_shared::dependency::DependencyDescriptor;
use web_framework_shared::argument_resolver::{ArgumentResolver, ResolveArguments};

use knockoff_logging::*;
use lazy_static::lazy_static;
use std::sync::Mutex;
use codegen_utils::project_directory;
import_logger_root!("lib.rs", concat!(project_directory!(), "/log_out/handler_mapping.log"));


pub struct HandlerMappingBuilder {
    controllers: Vec<ControllerBean>,
}

impl HandlerMappingBuilder {

    pub fn new(items: &mut ProfileTree) -> Self {
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

    fn filter_controller_beans(b: &&BeanDefinition) -> bool {
        b.struct_found.as_ref()
            .map(|s| SynHelper::get_attr_from_vec(
                &s.attrs,
                &vec!["controller", "rest_controller"])
            )
            .flatten()
            .is_some()
    }

    fn create_controller_beans(bean: &BeanDefinition) -> Vec<ControllerBean> {
        bean.traits_impl.iter()
            .flat_map(|b| Self::create_req_matcher_tuple(b))
            .flat_map(|autowire| Self::expand_autowire_for_items(autowire))
            .flat_map(|i| Self::create_controller_bean(i))
            .collect::<Vec<ControllerBean>>()
    }

    fn expand_autowire_for_items(autowire: (DependencyDescriptor, Vec<AntPathRequestMatcher>)) -> Vec<(DependencyDescriptor, Vec<AntPathRequestMatcher>, ImplItem)> {
        autowire.0.item_impl
            .as_ref()
            .map(|i| i.items.as_ref())
            .or(Some(&vec![]))
            .unwrap()
            .iter()
            .map(|i| (autowire.0.clone(), autowire.1.clone(), i.clone()))
            .collect::<Vec<(DependencyDescriptor, Vec<AntPathRequestMatcher>, ImplItem)>>()
    }

    fn create_controller_bean(i: (DependencyDescriptor, Vec<AntPathRequestMatcher>, ImplItem)) -> Vec<ControllerBean> {
        match i.2 {
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
        }
    }

    fn create_req_matcher_tuple(b: &DependencyDescriptor) -> Vec<(DependencyDescriptor, Vec<AntPathRequestMatcher>)> {
        b.item_impl.as_ref().map(|item| item.items.iter()
            .flat_map(|i| {
                match i {
                    ImplItem::Method(impl_item_method) => {
                        vec![(b.clone(), Self::create_request_matcher(&impl_item_method.attrs))]
                    }
                    _ => {
                        vec![]
                    }
                }
            })
            .collect::<Vec<(DependencyDescriptor, Vec<AntPathRequestMatcher>)>>())
            .or(Some(vec![]))
            .unwrap()
    }

    pub fn generate_token_stream(&self) -> TokenStream {
        let (to_match, split) = self.get_request_matcher_info();

        let (arg_idents, arg_types, arg_outputs) = self.parse_request_body_args();

        let (path_var_idents, path_var_names) = self.parse_path_variable_args();

        let method_logic_stmts = self.reparse_method_logic();

        let mut ts = quote! {

            use serde::{Serialize, Deserialize};
            use web_framework::web_framework::context::Context;
            use web_framework::web_framework::request_context::SessionContext;
            use web_framework_shared::request::WebRequest;
            use web_framework_shared::controller::{ContextData, Data, HandlerExecutionChain};
            use web_framework_shared::request::WebResponse;
            use web_framework_shared::EndpointMetadata;
            use web_framework_shared::Handler;
            use web_framework::web_framework::context::UserRequestContext;
            use web_framework::web_framework::context::RequestContextData;
            use web_framework_shared::controller::{HandlerExecutor, HandlerMethod, HandlerExecutorStruct};
            use web_framework_shared::matcher::{AntPathRequestMatcher,AntStringRequestMatcher};

            pub struct Dispatcher {
                handler_mapping: AttributeHandlerMapping
            }

            pub struct AttributeHandlerMapping {
                #(#arg_idents: Arc<HandlerExecutionChain<
                    UserRequestContext<#arg_types>,
                    RequestContextData<#arg_types, #arg_outputs>,
                    #arg_types,
                    #arg_outputs,
                    HandlerExecutorImpl<
                        UserRequestContext<#arg_types>,
                        RequestContextData<#arg_types, #arg_outputs>
                    >>>)*
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

                impl Handler<#arg_types, #arg_outputs, UserRequestContext<#arg_types>, RequestContextData<#arg_types, #arg_outputs>>
                for HandlerExecutorImpl<UserRequestContext<#arg_types>, RequestContextData<#arg_types, #arg_outputs>> {
                    fn do_action(
                        &self,
                        web_request: &WebRequest,
                        response: &mut WebResponse,
                        context: &RequestContextData<#arg_types, #arg_outputs>,
                        request_context: &mut Option<Box<UserRequestContext<#arg_types>>>
                    ) -> Option<#arg_outputs> {
                        if request_context.as_ref().is_none() {
                            let hm = HandlerMethod::new(UserRequestContext::new_default().into());
                            return self.execute_handler(hm, response, web_request);
                        }

                        let mut request_ctx_data: Option<Box<UserRequestContext<#arg_types>>> = None;
                        std::mem::swap(&mut request_ctx_data, request_context);
                        let hm = HandlerMethod::new(request_ctx_data.unwrap());
                        self.execute_handler(hm, response, web_request)
                    }

                    /**
                    For method level annotations (could also be done via Aspect though).
                    */
                    fn authentication_granted(&self, token: &Option<Box<UserRequestContext<#arg_types>>>) -> bool {
                        true
                    }

                    /**
                    determines if it matches endpoint, http method, etc.
                    */
                    fn matches(&self, endpoint_metadata: &EndpointMetadata) -> bool {
                        let mut to_match_vec = vec![];

                        #(
                            to_match_vec.push((#to_match, #split));
                        )*

                        if endpoint_metadata.matches_vec(to_match_vec)  {
                            return true;
                        }

                        false
                    }
                }

                impl HandlerExecutor<UserRequestContext<#arg_types>, RequestContextData<#arg_types, #arg_outputs>, #arg_types, #arg_outputs>
                for HandlerExecutorImpl<UserRequestContext<#arg_types>, RequestContextData<#arg_types, #arg_outputs>>
                {
                    fn execute_handler(
                        &self,
                        handler: HandlerMethod<UserRequestContext<#arg_types>>,
                        response: &mut WebResponse,
                        request: &WebRequest
                    ) -> Option<#arg_outputs> {
                        if handler.request_ctx_data.as_ref().is_none() {
                             return None
                        }
                        handler.request_ctx_data.unwrap().request
                            .map(|#arg_idents| {
                                 #(#method_logic_stmts)*
                            })
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

                        let handler_executor: Arc<HandlerExecutorStruct<
                            HandlerExecutorImpl<UserRequestContext<#arg_types>, RequestContextData<#arg_types, #arg_outputs>>,
                            UserRequestContext<#arg_types>,
                            RequestContextData<#arg_types, #arg_outputs>,
                            #arg_types,
                            #arg_outputs
                        >> = Arc::new(HandlerExecutorStruct {
                            handler_executor,
                            phantom_data_t: PhantomData::default(),
                            phantom_data_ctx: PhantomData::default(),
                            response: PhantomData::default(),
                            request: PhantomData::default()
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

        };
        ts.into()
    }

    // Has to be statements or else the {} will not allow the let statements
    fn reparse_method_logic(&self) -> Vec<Vec<&Stmt>> {
        let method_logic = self.controllers.iter()
            .map(|c| {
                log_message!("Here is the next block for controller bean: {}.", SynHelper::get_str(&c.method.block));
                c.method.block.stmts.iter().collect::<Vec<&Stmt>>()
            })
            .map(|f| f.clone())
            .collect::<Vec<Vec<&Stmt>>>();
        method_logic
    }

    fn parse_path_variable_args(&self) -> (Vec<Vec<Ident>>, Vec<Vec<String>>) {
        let path_var_idents = self.get_path_var_idents();
        let path_var_names = self.get_path_var_names();
        (path_var_idents, path_var_names)
    }

    // The path variables will be kept in a map, so you have to have the names to get them from
    // the map.
    fn get_path_var_names(&self) -> Vec<Vec<String>> {
        let path_var_names = self.controllers.iter()
            .flat_map(|c| c.arguments_resolved.iter()
                .filter(|a| {
                    a.path_variable_arguments.len() != 0
                })
                .map(|a| a.path_variable_arguments.clone())
                .map(|args| {
                    args.iter().map(|args| {
                        args.inner.name.clone()
                    }).collect::<Vec<String>>()
                })
            )
            .collect::<Vec<Vec<String>>>();
        path_var_names
    }

    fn get_path_var_idents(&self) -> Vec<Vec<Ident>> {
        let mut path_var_idents = self.controllers.iter()
            .flat_map(|c| c.arguments_resolved.iter()
                .map(|a| a.path_variable_arguments.clone())
                .map(|args| {
                    args.iter().flat_map(|args| {
                        if args.inner.name.len() != 0 {
                            return vec![Ident::new(args.inner.name.as_str(), Span::call_site())];
                        }
                        vec![]
                    }).collect::<Vec<Ident>>()
                })
            )
            .collect::<Vec<Vec<Ident>>>();
        path_var_idents
    }

    fn parse_request_body_args(&self) -> (Vec<Ident>, Vec<Type>, Vec<Type>) {
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
        let arg_idents = args.iter()
            .map(|i| i.0.clone())
            .collect::<Vec<Ident>>();
        let arg_types = args.iter()
            .map(|i| i.1.clone())
            .collect::<Vec<Type>>();
        (arg_idents, arg_types, args.iter().map(|i| i.2.clone()).collect::<Vec<Type>>())
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
        SynHelper::get_attr_from_vec(&attr, &vec!["get_mapping", "post_mapping"])
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


