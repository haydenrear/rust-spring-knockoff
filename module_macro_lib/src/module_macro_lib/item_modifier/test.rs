use quote::quote;
use syn::{parse, parse2, parse_macro_input, Stmt};
use module_macro_codegen::aspect::MethodAdviceAspectCodegen;

#[test]
fn test_get_proceed_ident() {
    let this = quote!{
        let this = "";
    };
    let parsed_stmt = parse2::<Stmt>(this.into());
    assert!(parsed_stmt.is_ok());
}