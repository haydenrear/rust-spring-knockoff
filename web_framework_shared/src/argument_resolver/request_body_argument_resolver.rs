use syn::{ImplItemMethod, Path};
use crate::argument_resolver::{NamedValueInfo, ResolveArguments};

pub struct RequestBodyArgumentResolver {
    inner: NamedValueInfo,
    request_serialize_type: syn::Type
}

impl ResolveArguments for RequestBodyArgumentResolver {
    fn resolve_argument_methods(method: &ImplItemMethod) -> Vec<Self> where Self: Sized {
        vec![]
    }
}
