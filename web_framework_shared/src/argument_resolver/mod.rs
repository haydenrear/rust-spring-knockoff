use std::ops::Deref;
use proc_macro2::Ident;
use syn::{FnArg, ImplItemMethod, Pat, PatType, ReturnType, Type};
use codegen_utils::syn_helper::SynHelper;
use crate::argument_resolver::path_variable_argument_resolver::PathVariableMethodArgument;
use crate::argument_resolver::query_param_argument_resolver::QueryParamMethodArgument;
use crate::argument_resolver::request_body_argument_resolver::RequestBodyArgumentResolver;

pub mod path_variable_argument_resolver;
pub mod query_param_argument_resolver;
pub mod request_body_argument_resolver;

pub struct ArgumentResolver {
    pub path_variable_arguments: Vec<PathVariableMethodArgument>,
    pub query_param_arguments: Vec<QueryParamMethodArgument>,
    pub request_body_arguments: Vec<RequestBodyArgumentResolver>
}

#[derive(Clone,Default)]
pub struct NamedValueInfo {
    pub name: String,
    pub required: bool,
    pub default_value: String,
    pub label: String,
    pub multi_valued: bool
}

pub trait ResolveArguments {

    fn resolve_argument_methods(method: &ImplItemMethod) -> Vec<Self>
    where Self: Sized;

    fn resolve_fn_arg_fn_arg_ident_tuple<'a>(attr_name: &str, method: &'a ImplItemMethod)
        -> Vec<(&'a PatType, String)>
        where Self: Sized
    {
        method.sig.inputs.iter().flat_map(|i| match i {
            FnArg::Receiver(_) => {
                vec![]
            }
            FnArg::Typed(typed_arg) => {
                SynHelper::get_attr_from_vec(&typed_arg.attrs, vec![attr_name])
                    .map(|attr_item| {
                        if attr_item.len() == 0 {
                            return SynHelper::get_fn_arg_ident_type(typed_arg)
                                .map(|t| vec![(typed_arg, t.0.to_string())])
                                .or(Some(vec![]))
                                .unwrap();
                        }
                        vec![(typed_arg, attr_item)]
                    })
                    .or(Some(vec![]))
                    .unwrap()
            }
        }).collect::<Vec<(&PatType, String)>>()
    }


    fn resolve_fn_arg_fn_output(method: &ImplItemMethod) -> Option<Type>
    {
        match &method.sig.output {
            ReturnType::Default => {
                None
            }
            ReturnType::Type(_, out) => {
                Some(out.deref().clone())
            }
        }
    }

    fn get_method_arg_ident(typed_arg: &PatType) -> Option<Ident> {
        match &typed_arg.pat.deref() {
            &Pat::Ident(pat) => {
                Some(pat.ident.clone())
            }
            _ => {
                None
            }
        }
    }
}

impl ResolveArguments for ArgumentResolver {
    fn resolve_argument_methods(method: &ImplItemMethod) -> Vec<Self>
        where Self: Sized
    {
        vec![Self {
            path_variable_arguments: PathVariableMethodArgument::resolve_argument_methods(method),
            query_param_arguments: QueryParamMethodArgument::resolve_argument_methods(method),
            request_body_arguments: RequestBodyArgumentResolver::resolve_argument_methods(method),
        }]
    }
}