use syn::Item;
use crate::parse_container::ParseContainer;

pub trait ParseContainerModifier {
    fn do_modify(items: &mut ParseContainer);
}

pub trait ParseContainerItemUpdater {
    fn parse_update(items: &mut Item, parse_container: &mut ParseContainer);
}

