use std::ops::Deref;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{parse_str, Attribute, Block, FnArg, ImplItem, ImplItemMethod, ItemImpl, Stmt, Type};

// Type alias for proc_macro2::TokenStream to distinguish from proc_macro::TokenStream
type TokenStream2 = proc_macro2::TokenStream;
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

    message_converters: Vec<MessageConverterBean>
}

impl HandlerMappingBuilder {

    pub fn new(items: &mut ProfileTree) -> Self {

        info!("Parsing controllers.");
        let controller_beans = Self::find_bean_definition(items, vec!["controller", "rest_controller"])
            .iter()
            .flat_map(|bean| Self::create_controller_beans(bean))
            .collect::<Vec<ControllerBean>>();
        
        info!("Parsing message converters.");
        
        let message_converters = Self::find_bean_definition(items, vec!["message_converter"])
            .iter()
            .flat_map(|bean| Self::create_message_converter_bean(bean))
            .collect::<Vec<MessageConverterBean>>();

        Self {
            controllers: controller_beans,
            message_converters: vec![]
        }

    }

    fn find_bean_definition<'a>(items: &'a mut ProfileTree, matchers: Vec<&str>) -> Vec<&'a BeanDefinition> {
        items.injectable_types.iter()
            .flat_map(|b| b.1.iter())
            .flat_map(|b| match b {
                BeanDefinitionType::Abstract { .. } => {
                    vec![]
                }
                BeanDefinitionType::Concrete { bean } => {
                    vec![bean]
                }
            })
            .filter(|b| {
                b.struct_found.as_ref()
                    .map(|s| SynHelper::get_attr_from_vec(
                        &s.attrs,
                        &matchers)
                    )
                    .flatten()
                    .is_some()
            })
            .collect()
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

    fn filter_message_converter_beans(b: &&BeanDefinition) -> bool {
        b.struct_found.as_ref()
            .map(|s| SynHelper::get_attr_from_vec(
                &s.attrs,
                &vec!["message_converter"])
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

    fn create_message_converter_bean(bean: &BeanDefinition) -> Vec<MessageConverterBean> {
        use syn::parse_str;
        use proc_macro2::TokenStream;
        
        let depth = bean.path_depth.clone();
        let struct_ty = bean.struct_found.as_ref().unwrap().ident.clone().to_string();

        let mut out = "".to_string();

        for d in depth {
            out = out + &d;
            out = out + "::";
        }

        out = out + &struct_ty;

        // Extract message converter attribute
        // Extract media_type and alias from the attribute
        // Try to extract request and response types from generics
        bean.traits_impl
            .iter()
            .filter(|t| t.item_impl.as_ref()
                .map(|i| {
                    info!("Testing {}", SynHelper::get_str(&i));
                    i
                })
                .map(|i| SynHelper::get_str(i.self_ty.deref()).ends_with("MessageConverter"))
                .or(Some(false))
                .unwrap())
            .flat_map(|dd| dd.item_impl.clone().into_iter())
            .flat_map(|item_impl| {

                let attr = SynHelper::get_attribute_from_vec(&item_impl.attrs , &vec!["message_converter"]);

                let (media_type, alias) = if let Some(attr_meta) = attr {
                    match attr_meta.parse_meta() {
                        Ok(syn::Meta::List(list)) => {
                            let media_type = list.nested.iter().find_map(|meta| Self::extract_str_from_meta(meta, "media_type"));
                            let alias = list.nested.iter().find_map(|meta| Self::extract_str_from_meta(meta, "alias"));

                            (media_type, alias)
                        },
                        _ => (None, None)
                    }
                } else {
                    (None, None)
                };


                let (request_type, response_type) = if let Some(generics) = &item_impl.generics.params.iter().next() {
                    if let syn::GenericParam::Type(type_param) = generics {
                        // If there's at least one generic, assume it's the request type
                        let req_type = match parse_str::<TokenStream>(&type_param.ident.to_string()) {
                            Ok(ts) => Some(ts),
                            Err(e) => {
                                error!("Failed to parse request type: {}", e);
                                None
                            }
                        };

                        // If there's a second generic, assume it's the response type
                        let resp_type = if let Some(second) = item_impl.generics.params.iter().nth(1) {
                            if let syn::GenericParam::Type(type_param) = second {
                                match parse_str::<TokenStream>(&type_param.ident.to_string()) {
                                    Ok(ts) => Some(ts),
                                    Err(e) => {
                                        error!("Failed to parse response type: {}", e);
                                        None
                                    }
                                }
                            } else {
                                None
                            }
                        } else {
                            // If no second generic, assume response is same as request
                            req_type.clone()
                        };

                        (req_type, resp_type)
                    } else {
                        (None, None)
                    }
                } else {
                    (None, None)
                };

                parse_str::<syn::Path>(&out)
                    .map(|p| {
                        info!("Created path for message converter: {}", SynHelper::get_str(&p));
                        MessageConverterBean {
                            converter_path: p,
                            request_type,
                            response_type,
                            media_type,
                            alias,
                        }
                    })
                    .or_else(|err| {
                        error!("Found error: {}!", err);
                        Err(err)
                    })
                    .ok()
                    .into_iter()
            })
            .map(|m| {
                info!("Found message converter bean: {}, {}", SynHelper::get_str(&m.converter_path), &m.media_type.as_ref().unwrap_or(&"".to_string()));
                m
            })
            .collect()


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

    // Helper function to extract a string literal from a meta item
    fn extract_str_from_meta(meta: &syn::NestedMeta, name: &str) -> Option<String> {
        if let syn::NestedMeta::Meta(syn::Meta::NameValue(nv)) = meta {
            if nv.path.is_ident(name) {
                if let syn::Lit::Str(lit_str) = &nv.lit {
                    return Some(lit_str.value());
                }
            }
        }
        None
    }

    fn generate_message_converter_tokens(&self) -> Option<TokenStream> {
        if self.message_converters.is_empty() {
            return None;
        }

        use proc_macro2::Span;
        use syn::LitStr;
        use std::collections::HashMap;
        use quote::{format_ident, quote};

        // Group converters by request/response types
        let mut converter_groups: HashMap<(String, String), Vec<&MessageConverterBean>> = HashMap::new();
        
        for converter in &self.message_converters {
            if let (Some(req_type), Some(resp_type)) = (&converter.request_type, &converter.response_type) {
                let req_key = req_type.to_string();
                let resp_key = resp_type.to_string();
                converter_groups.entry((req_key, resp_key))
                    .or_insert_with(Vec::new)
                    .push(converter);
            }
        }

        if converter_groups.is_empty() {
            return None;
        }

        // Build the macro input structure
        let mut group_tokens = Vec::new();
        
        for ((req_key, resp_key), converters) in converter_groups {
            let mut converter_tokens = Vec::new();
            
            for converter in converters.iter() {
                if let (Some(media_type), Some(alias)) = (&converter.media_type, &converter.alias) {
                    let converter_path = &converter.converter_path;
                    let req_type = converter.request_type.as_ref().unwrap();
                    let resp_type = converter.response_type.as_ref().unwrap();
                    let media_type_lit = LitStr::new(media_type, Span::call_site());
                    let alias_str = format_ident!("{}", alias);
                    
                    converter_tokens.push(quote! {
                        (#media_type_lit as #alias_str => #converter_path<#req_type, #resp_type>)
                    });
                }
            }
            
            if !converter_tokens.is_empty() {
                let converters_tuple = quote! { (#(#converter_tokens),*) };
                let req_type = converters[0].request_type.as_ref().unwrap();
                let resp_type = converters[0].response_type.as_ref().unwrap();
                group_tokens.push(quote! {
                    #converters_tuple ===> (#req_type => #resp_type)
                });
            }
        }

        if group_tokens.is_empty() {
            return None;
        }

        // Build the final macro call
        Some(quote! {
            create_delegating_message_converters!((
                #(#group_tokens),*
            ) => DelegatingMessageConverter);
        })
    }

    pub fn generate_token_stream(&self) -> TokenStream {
        // Generate message converter tokens if needed
        let message_converter_tokens = self.generate_message_converter_tokens();
        
        let (to_match, split) = self.get_request_matcher_info();

        let (arg_idents, arg_types, arg_outputs) = self.parse_request_body_args();
        
        // Include the message converter macro if we have converters
        let converter_tokens = message_converter_tokens.unwrap_or_else(|| quote! {});

        let (path_var_idents, path_var_names) = self.parse_path_variable_args();

        let method_logic_stmts = self.reparse_method_logic();

        // Add imports for message converters
        let ts = quote! {
            use web_framework::web_framework::convert::*;
            use web_framework::create_delegating_message_converters;
            use web_framework::provide_default_message_converters;

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
                        mut handler: HandlerMethod<UserRequestContext<#arg_types>>,
                        response: &mut WebResponse,
                        request: &WebRequest
                    ) -> Option<#arg_outputs> {
                        if handler.request_ctx_data.as_ref().is_none() {
                            let mut req = UserRequestContext::default();
                            req.request = Some(#arg_types::default());
                            req.request
                                .map(|#arg_idents| {
                                     #(#method_logic_stmts)*
                                })
                        } else {
                            println!("Not null!");
                            handler.request_ctx_data.unwrap().request
                                .map_or_else(|| {
                                    println!("Was null!");
                                    let #arg_idents = #arg_types::default();
                                     #(#method_logic_stmts)*
                                }, |#arg_idents| {
                                     #(#method_logic_stmts)*
                                })
                                .into()
                        }
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

            // Include the message converter tokens if we have any
            // provide_default_message_converters!();
            // #converter_tokens


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

struct MessageConverterBean {
    converter_path: syn::Path,
    request_type: Option<TokenStream>,
    response_type: Option<TokenStream>,
    media_type: Option<String>,
    alias: Option<String>,
}



impl Default for MessageConverterBean {
    fn default() -> Self {
        Self {
            converter_path: parse_str::<syn::Path>("DefaultMessageConverter").unwrap(),
            request_type: None,
            response_type: None,
            media_type: None,
            alias: None,
        }
    }
}
