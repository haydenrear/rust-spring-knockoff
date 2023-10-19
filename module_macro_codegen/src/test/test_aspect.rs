use std::env;
use quote::ToTokens;
use syn::Item;
use crate::parser::{CodegenItem, LibParser};


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
