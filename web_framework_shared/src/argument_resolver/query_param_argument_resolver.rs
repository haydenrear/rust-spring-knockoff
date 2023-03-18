use syn::ImplItemMethod;
use crate::argument_resolver::{NamedValueInfo, ResolveArguments};

pub struct QueryParamMethodArgument {
    pub inner: NamedValueInfo
}

impl ResolveArguments for QueryParamMethodArgument {
    fn resolve_argument_methods(method: &ImplItemMethod) -> Vec<Self> where Self: Sized {
        Self::resolve_fn_arg_fn_arg_ident_tuple("request_param", method)
            .iter()
            .map(|method_arg_name| {
                Self {
                    inner: NamedValueInfo {
                        name: method_arg_name.1.clone(),
                        required: false,
                        default_value: "".to_string(),
                        label: "request param".to_string(),
                        multi_valued: false,
                    }
                }
            })
            .collect()
    }
}