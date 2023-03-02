use crate::module_macro_lib::aspect::AspectParser;

#[test]
fn get_aspect() {
   let codegen_items = AspectParser::parse_aspects();
    assert!(codegen_items.is_some());
    assert_eq!(codegen_items.unwrap().codegen.len(), 1);
}