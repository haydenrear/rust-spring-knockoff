use syn::{ImplItem, ImplItemMethod, Item, Pat};
use codegen_utils::syn_helper::SynHelper;
use crate::argument_resolver::path_variable_argument_resolver::PathVariableMethodArgument;
use crate::argument_resolver::query_param_argument_resolver::QueryParamMethodArgument;
use crate::argument_resolver::request_body_argument_resolver::RequestBodyArgumentResolver;
use crate::argument_resolver::ResolveArguments;

#[test]
fn test_request_body_fn_arg_resolver() {
    let resolved = parse_impl_item_method("codegen_resources/test_argument_resolver_request_body.rs")
        .iter()
        .flat_map(|i| RequestBodyArgumentResolver::resolve_argument_methods(i))
        .collect::<Vec<RequestBodyArgumentResolver>>();
    assert_eq!(resolved.len(), 2);
    assert_for_all(resolved, &|r| r.inner.name == "test_request_body");
}

#[test]
fn test_request_body_path_variable() {
    let resolved = parse_impl_item_method("codegen_resources/test_argument_resolver_path_variable.rs")
        .iter()
        .flat_map(|i| PathVariableMethodArgument::resolve_argument_methods(i))
        .collect::<Vec<PathVariableMethodArgument>>();
    assert_eq!(resolved.len(), 2);
    assert_for_all(resolved, &|r| r.inner.name == "two");
}

#[test]
fn test_request_body_request_param() {
    let resolved = parse_impl_item_method("codegen_resources/test_argument_resolver_request_param.rs")
        .iter()
        .flat_map(|i| QueryParamMethodArgument::resolve_argument_methods(i))
        .collect::<Vec<QueryParamMethodArgument>>();
    assert_eq!(resolved.len(), 2);
    assert_for_all(resolved, &|r| r.inner.name == "test_request_param");
}

fn assert_for_all<T>(items: Vec<T>, to_do: &dyn Fn(&T) -> bool) {
    items.iter().for_each(|i| assert!(to_do(i)));
}

fn parse_impl_item_method(path: &str) -> Vec<ImplItemMethod> {
    SynHelper::open_from_base_dir(path)
        .items.iter()
        .flat_map(|i| match i {
            Item::Impl(impl_item) => {
                impl_item.items.iter().flat_map(|i| match i {
                    ImplItem::Method(impl_item_method) => {
                        vec![impl_item_method.clone()]
                    }
                    _ => {
                        vec![]
                    }
                }).collect::<Vec<ImplItemMethod>>()
            }
            _ => {
                vec![]
            }
        })
        .collect()
}