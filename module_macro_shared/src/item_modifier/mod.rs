use syn::Item;
use crate::parse_container::ParseContainer;

pub trait ItemModifier {
    fn modify_item(&self, parse_container: &mut ParseContainer, item: &mut Item, path_depth: Vec<String>);
    fn supports_item(&self, item: &Item) -> bool;
}

#[derive(Default)]
pub struct DelegatingItemModifier {
    modifiers: Vec<Box<dyn ItemModifier>>
}

impl DelegatingItemModifier {
    pub fn new(modifiers: Vec<Box<dyn ItemModifier>>) -> Self {
        Self {
            modifiers
        }
    }
}

impl ItemModifier for DelegatingItemModifier {

    fn modify_item(&self, parse_container: &mut ParseContainer, item: &mut Item, path_depth: Vec<String>) {
        let mut path_depth = path_depth.clone();
        self.modifiers.iter().for_each(|f| {
            if f.supports_item(&item) {
                f.modify_item(parse_container, item, path_depth.clone());
            }
        });
        match item {
            Item::Mod(ref mut item_mod) => {
                let mod_ident = item_mod.ident.to_string().clone();
                if !path_depth.contains(&mod_ident) {
                    path_depth.push(mod_ident);
                }
                item_mod.content.iter_mut().for_each(|c| {
                    for item in c.1.iter_mut() {
                        self.modify_item(parse_container, item, path_depth.clone())
                    }
                });
            }
            _ => {}
        }
    }

    fn supports_item(&self, item: &Item) -> bool {
        self.modifiers.iter().any(|f| f.supports_item(item))
    }
}
