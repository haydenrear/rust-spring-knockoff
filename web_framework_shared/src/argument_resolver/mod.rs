use syn::ImplItemMethod;
use crate::argument_resolver::path_variable_argument_resolver::PathVariableMethodArgument;
use crate::argument_resolver::query_param_argument_resolver::QueryParamMethodArgument;
use crate::argument_resolver::request_body_argument_resolver::RequestBodyArgumentResolver;

pub mod path_variable_argument_resolver;
pub mod query_param_argument_resolver;
pub mod request_body_argument_resolver;

pub struct ArgumentResolver {
    path_variable_arguments: Vec<PathVariableMethodArgument>,
    query_param_arguments: Vec<QueryParamMethodArgument>,
    request_body_arguments: Vec<RequestBodyArgumentResolver>
}

pub struct NamedValueInfo {
    name: String,
    required: bool,
    default_value: String,
    label: String,
    multi_valued: bool
}

pub trait ResolveArguments {
    fn resolve_argument_methods(method: &ImplItemMethod) -> Vec<Self>
    where Self: Sized;
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