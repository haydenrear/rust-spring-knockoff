use std::fmt::Debug;
use std::fs::File;
use std::path::Path;
use quote::ToTokens;

use syn::Item;

use codegen_utils::project_directory;
use codegen_utils::syn_helper::SynHelper;
use module_macro_shared::{get_test_module_parser, ItemParser, ParseContainer, ProfileProfileTreeModifier, ProfileTreeBuilder, ProfileTreeModifier};
use module_macro_shared::item_impl_parser::ItemImplParser;
use module_macro_shared::item_struct_parser::ItemStructParser;

use crate::aspect_knockoff_provider::aspect_item_modifier::AspectParser;
use crate::aspect_knockoff_provider::aspect_parse_provider::ParsedAspects;
use crate::aspect_knockoff_provider::aspect_ts_generator::AspectGenerator;

#[test]
fn test_parse_aspect() {
    let mut parse_container = ParseContainer::default();
    let multiple = Path::new(project_directory!()).join("codegen_resources").join("aspect_test.rs");

    let mut read = SynHelper::parse_syn_file(&mut File::options().read(true).open(multiple).unwrap()).unwrap();

    let mut module_parser = get_test_module_parser();

    read.items.iter_mut().for_each(|f| { {
        match f {
            Item::Struct(s) => {
                ItemStructParser::parse_item(&mut parse_container, s, vec![], &mut module_parser);
            }
            Item::Impl(i) => {
                ItemImplParser::parse_item(&mut parse_container, i, vec![], &mut module_parser);
            }
            _ => {}
        }
    } });

    read.items.iter_mut().for_each(|f| {
        ParsedAspects::parse_update(f, &mut parse_container)
    });

    read.items.iter_mut().for_each(|f| {
        AspectParser::modify_item(&mut parse_container, f, vec![]);
    });

    let p = Box::new(ProfileProfileTreeModifier::new(&parse_container.injectable_types_builder));
    let mut profile_tree = ProfileTreeBuilder::build_profile_tree(&mut parse_container.injectable_types_builder, vec![p], &mut parse_container.provided_items);

    let generator = AspectGenerator::new(&mut profile_tree);
    let a = generator.generate_token_stream();

    println!("{}", SynHelper::get_str(a));

}

