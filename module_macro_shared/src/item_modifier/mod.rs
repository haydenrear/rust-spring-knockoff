use syn::Item;
use crate::parse_container::ParseContainer;

pub trait ItemModifier {

    fn modify_item(parse_container: &mut ParseContainer, item: &mut Item,
                   path_depth: Vec<String>);

    fn new() -> Self;

    fn supports_item(item: &Item) -> bool;

}
