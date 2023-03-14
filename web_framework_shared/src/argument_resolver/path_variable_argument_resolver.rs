use syn::{ImplItemMethod, Path};
use crate::argument_resolver::{NamedValueInfo, ResolveArguments};

pub struct PathVariableMethodArgument {
    inner: NamedValueInfo
}

impl ResolveArguments for PathVariableMethodArgument {
    fn resolve_argument_methods(method: &ImplItemMethod) -> Vec<Self> where Self: Sized {
        vec![]
    }
}