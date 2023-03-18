use syn::{Attribute, ImplItemMethod, Path};
use codegen_utils::syn_helper::SynHelper;
use crate::argument_resolver::{NamedValueInfo, ResolveArguments};

pub struct PathVariableMethodArgument {
    pub inner: NamedValueInfo
}

impl ResolveArguments for PathVariableMethodArgument {
    fn resolve_argument_methods(method: &ImplItemMethod) -> Vec<Self> where Self: Sized {
        Self::resolve_fn_arg_fn_arg_ident_tuple("path_variable", method)
            .iter()
            .map(|method_arg_name| {
                Self {
                    inner: NamedValueInfo {
                        name: method_arg_name.1.clone(),
                        required: false,
                        default_value: "".to_string(),
                        label: "path variable".to_string(),
                        multi_valued: false,
                    }
                }
            })
            .collect()
    }
}
