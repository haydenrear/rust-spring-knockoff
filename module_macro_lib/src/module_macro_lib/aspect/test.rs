use module_macro_codegen::aspect::AspectParser;

#[test]
fn get_aspect() {
   let codegen_items = AspectParser::parse_aspects();
    assert_ne!(codegen_items.aspects.len(), 0);
}