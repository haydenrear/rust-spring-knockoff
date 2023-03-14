use std::env;
use quote::ToTokens;
use syn::Item;
use crate::aspect::{AspectParser, MethodAdviceAspectCodegen};
use crate::parser::{CodegenItem, LibParser};

#[test]
fn test_parse_aspect() {
    let mut aspects = AspectParser::parse_aspects();
    assert_eq!(aspects.aspects.len(), 1);
    let mut first = aspects.aspects.remove(0);
    assert_eq!(first.method_advice_aspects.len(), 1);
    let method_advice = first.method_advice_aspects.remove(0);
    assert!(!method_advice.before_advice.clone().unwrap().to_token_stream().to_string().as_str().contains("proceed"));
    assert!(!method_advice.after_advice.clone().unwrap().to_token_stream().to_string().as_str().contains("proceed"));
    assert!(method_advice.item.is_some());
    match method_advice.item.unwrap() {
        Item::Fn(fn_found) => {
            assert_eq!(fn_found.sig.ident.to_string().clone(), "do_aspect");
        }
        _ => {
            assert!(false)
        }
    }
}

#[test]
fn test_lib_parse() {
    let knockoff_files = env::var("AUG_FILE");
    assert!(knockoff_files.is_ok());
    let parsed = LibParser::parse_codegen_items(knockoff_files.unwrap().as_str());
    assert_ne!(parsed.len(), 0);
    parsed.iter().for_each(|f| {
        match f.get_unique_id().as_str()  {
            "" => {

            }
            _ => {
                // assert!(false)
            }
        }
    })
}
