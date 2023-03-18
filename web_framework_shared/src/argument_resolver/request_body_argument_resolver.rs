use std::ops::Deref;
use proc_macro2::Ident;
use syn::{FnArg, ImplItemMethod, Pat, Path, PatType};
use codegen_utils::syn_helper::SynHelper;
use crate::argument_resolver::{NamedValueInfo, ResolveArguments};

pub struct RequestBodyArgumentResolver {
    pub inner: NamedValueInfo,
    pub request_serialize_type: syn::Type,
    pub output_type: Option<syn::Type>
}

impl ResolveArguments for RequestBodyArgumentResolver {
    fn resolve_argument_methods(method: &ImplItemMethod) -> Vec<Self>
        where Self: Sized
    {
        Self::resolve_fn_arg_fn_arg_ident_tuple("request_body", method)
            .iter()
            .map(|method_arg_name| {
                Self {
                    inner: NamedValueInfo {
                        name: method_arg_name.1.clone(),
                        required: false,
                        default_value: "".to_string(),
                        label: "request body".to_string(),
                        multi_valued: false,
                    },
                    request_serialize_type: method_arg_name.0.ty.deref().clone(),
                    output_type: Self::resolve_fn_arg_fn_output(method),
                }
            })
            .collect()
    }
}

