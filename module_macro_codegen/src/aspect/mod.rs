use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{Attribute, Item, ItemFn};
use crate::parser::CodegenItem;

#[derive(Clone, Default)]
pub struct MethodAdviceAspect {
    default: Option<TokenStream>,
    item: Option<Item>,
    before_advice: TokenStream,
    after_advice: TokenStream,
    aspect_matcher_string: String
}

impl MethodAdviceAspect {
    pub(crate) fn create_aspect_matcher(&self) -> AspectMatcher {
        Self::create_aspect(&self.aspect_matcher_string)
    }

    pub(crate) fn create_aspect(aspect_matcher: &str) -> AspectMatcher {
        let mut paths = aspect_matcher.split(".")
            .map(|s| s.to_string())
            .collect::<Vec<String>>();

        let aspect_matcher_path_len = paths.len();
        let mut struct_path = paths.remove(aspect_matcher_path_len - 1);
        AspectMatcher::new(paths.join("."), struct_path)
    }
}

/// Aspect matcher matches all structs/impls in particular modules and packages, and allows for
/// matching based on the struct name.
pub(crate) struct AspectMatcher {
    module_path: String,
    struct_path: String
}

impl AspectMatcher {
    fn new(module_path: String, struct_path: String) -> Self {
        Self {
            module_path, struct_path
        }
    }
}

impl CodegenItem for MethodAdviceAspect {

    fn supports_item(item: &Item) -> bool where Self: Sized {
        match item {
            Item::Fn(item_fn) => {
                Self::is_aspect(&item_fn.attrs)
            }
            Item::Mod(mod_aspect) => {
                Self::is_aspect(&mod_aspect.attrs)
            }
            _ => {
                false
            }
        }
    }

    fn supports(&self, item: &Item) -> bool {
        todo!()
    }

    fn get_codegen(&self) -> Option<String> {
        todo!()
    }

    fn default_codegen(&self) -> String {
        let ts = quote! {
            #[derive(Parse, Default, Clone, Debug)]
            pub struct AspectItem;

            impl AspectItem {

            }

        };
        ts.to_string()
    }

    fn clone_dyn_codegen(&self) -> Box<dyn CodegenItem> {
        todo!()
    }

    fn get_unique_id(&self) -> String {
        todo!()
    }
}

impl MethodAdviceAspect {
    fn is_aspect(vec: &Vec<Attribute>) -> bool {
        vec.iter().any(|attr| attr.to_token_stream().to_string().as_str().contains("aspect"))
    }
}

#[cfg(test)]
mod test {
    use crate::aspect::MethodAdviceAspect;

    #[test]
    fn test_match() {
        let aspect = MethodAdviceAspect::create_aspect("hello.one.two.three.*");
        assert_eq!(aspect.struct_path, "*");
        assert_eq!(aspect.module_path, "hello.one.two.three");
    }
}
