use syn::ImplItemMethod;
use crate::argument_resolver::{NamedValueInfo, ResolveArguments};

pub struct QueryParamMethodArgument {
    inner: NamedValueInfo
}

impl ResolveArguments for QueryParamMethodArgument {
    fn resolve_argument_methods(method: &ImplItemMethod) -> Vec<Self> where Self: Sized {
        vec![]
    }
}