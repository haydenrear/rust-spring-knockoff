use quote::{quote, ToTokens};
use syn::{parse2, Type};
use crate::module_macro_lib::bean_parser::bean_dependency_path_parser::BeanDependencyPathParser;
use crate::module_macro_lib::module_tree::{BeanPath, BeanPathParts};

struct One;

#[test]
fn test_arc_mutex_bean_path_parsing() {
    let mutex_type = quote! {
        Mutex<One>
    };
    let arc_mutex_type = quote! {
        Arc<Mutex<One>>
    };

    let parsed_mutex = parse2::<syn::TypePath>(mutex_type);
    let parsed_arc_mutex= parse2::<syn::TypePath>(arc_mutex_type);

    assert!(parsed_mutex.is_ok());
    assert!(parsed_arc_mutex.is_ok());

    let parsed_arc_mutex = BeanDependencyPathParser::parse_type_path(parsed_arc_mutex.unwrap());
    let parsed_mutex = BeanDependencyPathParser::parse_type_path(parsed_mutex.unwrap());

    assert_eq!(parsed_mutex.path_segments.len(), 1);
    assert_eq!(parsed_arc_mutex.path_segments.len(), 2);

    let path = &parsed_mutex.path_segments[0];
    match path {
        BeanPathParts::MutexType {mutex_type_inner_type, outer_type} => {
            assert_eq!(mutex_type_inner_type.to_token_stream().to_string(), "One");
            assert_ne!(outer_type.segments.len(), 0);
            outer_type.segments.iter().any(|seg| seg.to_token_stream().to_string().contains("Mutex"));
            outer_type.segments.iter().any(|seg| seg.to_token_stream().to_string().contains("One"));
        }
        _ => {
            assert!(false);
        }
    }

    match &parsed_arc_mutex.path_segments.clone()[..] {
        [ BeanPathParts::ArcMutexType{ arc_mutex_inner_type, outer_type: out}, BeanPathParts::MutexType { mutex_type_inner_type, outer_type} ] => {
            assert_eq!(mutex_type_inner_type.to_token_stream().to_string(), "One");
            assert_ne!(outer_type.segments.len(), 0);
            out.segments.iter().any(|seg| seg.to_token_stream().to_string().contains("Arc"));
            out.segments.iter().any(|seg| seg.to_token_stream().to_string().contains("Mutex"));
            out.segments.iter().any(|seg| seg.to_token_stream().to_string().contains("One"));

            outer_type.segments.iter().any(|seg| seg.to_token_stream().to_string().contains("Mutex"));
            outer_type.segments.iter().any(|seg| seg.to_token_stream().to_string().contains("One"));
            outer_type.segments.iter().all(|seg| !seg.to_token_stream().to_string().contains("Arc"));

            assert!(arc_mutex_inner_type.to_token_stream().to_string().contains("Mutex"));
            assert!(arc_mutex_inner_type.to_token_stream().to_string().contains("One"));
            assert!(!arc_mutex_inner_type.to_token_stream().to_string().contains("Arc"));

        }
        _ => {
            assert!(false)
        }
    }

    let one_type = quote! {
        One
    };
    let two_type = quote! {
        Two
    };

    let parsed_one = parse2::<Type>(one_type);
    assert!(parsed_one.is_ok());
    let parsed_two = parse2::<Type>(two_type);
    assert!(parsed_two.is_ok());

    assert!(parsed_arc_mutex.bean_path_part_matches(&parsed_one.unwrap()));
    assert!(!parsed_arc_mutex.bean_path_part_matches(&parsed_two.unwrap()));

}