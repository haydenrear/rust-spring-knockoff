use syn::Item;
use crate::parse_container::ParseContainer;

/// After the ParseContainerItemUpdater updater runs as the container is loaded with beans, another
/// pass is done with the container, and the Items and the ParseContainer can be updated. All
/// beans will be loaded but some beans will have been updated in the container for this stage,
/// as it is a serially completed stage.
pub trait ItemModifier {

    fn modify_item(parse_container: &mut ParseContainer, item: &mut Item,
                   path_depth: Vec<String>);

    fn new() -> Self;

    fn supports_item(item: &Item) -> bool;

}

#[derive(Default)]
pub struct DefaultItemModifier;
impl ItemModifier for DefaultItemModifier {
    fn modify_item(parse_container: &mut ParseContainer, item: &mut Item, path_depth: Vec<String>) {
    }

    fn new() -> Self { Self {} }

    fn supports_item(item: &Item) -> bool {
        false
    }
}