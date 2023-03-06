use syn::{Item, Path, PathArguments, PathSegment, Type};
use proc_macro2::TokenStream;
use quote::quote;

#[cfg(test)]
pub mod test;

pub trait AspectMatcher {
    fn does_match(&self, item: Item, package: AspectAwarePackage) -> bool;
}

pub struct ProceedingJoinPoint;

pub enum JoinPoint<T, ARGS> {
    ProceedingJoinPointReturn(Box<dyn ProceedingJoinPointReturn<T, ARGS>>),
    ProceedingJoinPointReturnNoArgs(Box<dyn ProceedingJoinPointReturnNoArgs<T, ARGS>>),
    ProceedingJoinPointNoReturnArgs(Box<dyn ProceedingJoinPointNoReturnArgs<ARGS>>)
}

pub trait ProceedingJoinPointReturn<T, ARGS> {
    fn proceed(&self, args: ARGS) -> T;
}

pub trait ProceedingJoinPointReturnNoArgs<T, ARGS> {
    fn proceed(&self, args: ARGS) -> T;
}

pub trait ProceedingJoinPointNoReturnArgs<ARGS> {
    fn proceed(&self, args: ARGS);
}

pub struct AspectAwarePackage {
    /// module path with dots, such as aspect.another_module
    module_path: String,
    /// original path
    path: Path

}

impl AspectAwarePackage {
    pub fn new(path: &Path) -> Self {
        Self {
            module_path: Self::get_module_path(&path),
            path: path.clone()
        }
    }

    fn get_module_path(path: &Path) -> String {
        let mut path_str = "".to_string();
        for path_item in path.segments.iter() {
            match path_item.arguments {
                PathArguments::None => {
                    path_str += path_item.ident.to_string().as_str()
                }
                PathArguments::AngleBracketed(_) => {
                }
                PathArguments::Parenthesized(_) => {}
            }
        }
        path_str
    }
}
