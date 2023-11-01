use quote::{quote, ToTokens};
use syn::{parse2, Type};
use codegen_utils::syn_helper::SynHelper;
use crate::module_macro_lib::bean_parser::bean_dependency_path_parser::BeanDependencyPathParser;
use module_macro_shared::bean::BeanPathParts;

struct One;
trait OneTrait {}
impl OneTrait for One {}

#[test]
fn test_arc_mutex_bean_path_parsing() {
    let one_type = quote! {
        One
    };
    let one_dyn_type = quote! {
        dyn OneTrait
    };
    let mutex_type = quote! {
        Mutex<One>
    };
    let arc_mutex_type = quote! {
        Arc<Mutex<One>>
    };
    let dyn_type = quote! {
        Arc<dyn OneTrait>
    };
    let dyn_mutex_type = quote! {
        Arc<Mutex<Box<dyn OneTrait>>>
    };

    let one_type = parse2::<syn::TypePath>(one_type);
    let one_dyn_type = parse2::<syn::Type>(one_dyn_type);
    let parsed_mutex = parse2::<syn::TypePath>(mutex_type);
    let parsed_arc_mutex= parse2::<syn::TypePath>(arc_mutex_type);
    let parsed_dyn_type = parse2::<syn::TypePath>(dyn_type);
    let parsed_dyn_mutex_type= parse2::<syn::TypePath>(dyn_mutex_type);

    assert!(one_type.is_ok());
    assert!(one_dyn_type.is_ok());
    assert!(parsed_mutex.is_ok());
    assert!(parsed_arc_mutex.is_ok());
    assert!(parsed_dyn_type.is_ok());
    assert!(parsed_dyn_mutex_type.is_ok());


    let parsed_one_type = BeanDependencyPathParser::parse_type_path(one_type.unwrap());
    let parsed_one_dyn_type = BeanDependencyPathParser::parse_type(one_dyn_type.unwrap());
    let parsed_arc_mutex = BeanDependencyPathParser::parse_type_path(parsed_arc_mutex.unwrap());
    let parsed_mutex = BeanDependencyPathParser::parse_type_path(parsed_mutex.unwrap());
    let parsed_dyn_typ = BeanDependencyPathParser::parse_type_path(parsed_dyn_type.unwrap());
    let parsed_dyn_mutex_type = BeanDependencyPathParser::parse_type_path(parsed_dyn_mutex_type.unwrap());

    assert_eq!(parsed_one_type.path_segments.len(), 1);
    assert_eq!(parsed_one_dyn_type.unwrap().path_segments.len(), 1);
    assert_eq!(parsed_mutex.path_segments.len(), 1);
    assert_eq!(parsed_arc_mutex.path_segments.len(), 2);
    assert_eq!(parsed_dyn_typ.path_segments.len(), 1);
    assert_eq!(parsed_dyn_mutex_type.path_segments.len(), 3);

    let inner_one = quote! { One };
    let inner_dyn = quote! { dyn OneTrait };

    let one_found = parse2::<Type>(inner_one).unwrap();
    let dyn_found = parse2::<Type>(inner_dyn).unwrap();

    assert_eq!(SynHelper::get_str(&one_found), SynHelper::get_str(&parsed_one_type.get_inner_type().unwrap()));
    assert_eq!(SynHelper::get_str(&one_found), SynHelper::get_str(&parsed_arc_mutex.get_inner_type().unwrap()));
    assert_eq!(SynHelper::get_str(&one_found), SynHelper::get_str(&parsed_mutex.get_inner_type().unwrap()));
    assert_eq!(SynHelper::get_str(&dyn_found), SynHelper::get_str(&parsed_dyn_typ.get_inner_type().unwrap()));
    assert_eq!(SynHelper::get_str(&dyn_found), SynHelper::get_str(&parsed_dyn_mutex_type.get_inner_type().unwrap()));

    let path = &parsed_mutex.path_segments[0];
    match path {
        BeanPathParts::MutexType { inner_ty: mutex_type_inner_type, outer_path: outer_type } => {
            assert_eq!(mutex_type_inner_type.to_token_stream().to_string(), "One");
            assert_ne!(outer_type.segments.len(), 0);
            outer_type.segments.iter()
                .any(|seg| seg.to_token_stream().to_string().contains("Mutex"));
            outer_type.segments.iter()
                .any(|seg| seg.to_token_stream().to_string().contains("One"));
        }
        _ => {
            assert!(false);
        }
    }

    match &parsed_arc_mutex.path_segments.clone()[..] {
        [ BeanPathParts::ArcMutexType{ inner_ty: arc_mutex_inner_type, outer_path: out}, BeanPathParts::MutexType { inner_ty: mutex_type_inner_type, outer_path: outer_type } ] => {
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
