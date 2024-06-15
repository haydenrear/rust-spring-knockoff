use syn::{Ident, Item, ItemMod, token, Visibility};
use codegen_utils::FlatMapOptional;
use codegen_utils::syn_helper::SynHelper;
use crate::module_locator::{get_module_from_module_name, is_in_line_module};
use codegen_utils::{env, project_directory};
use knockoff_logging::*;
use lazy_static::lazy_static;
use std::sync::Mutex;
use crate::logger_lazy;
import_logger!("module_iterator.rs");

pub struct ModuleIterator {
    iter: Box<dyn Iterator<Item = Item>>
}

impl ModuleIterator {
    pub fn new(item_mod: &mut ItemMod) -> ModuleIterator {
        item_mod.to_owned().content
            .map(|(_, ct)| Self {iter: Box::new(ct.into_iter())})
            .or(Some(ModuleIterator {iter: Box::new(vec![].into_iter())}))
            .unwrap()
    }
}

impl Iterator for ModuleIterator {
    type Item = ItemMod;

    fn next(&mut self) -> Option<ItemMod> {
        loop {
            match self.iter.next() {
                Some(item) => match item {
                    // if in-line, return it, otherwise get the file, load it, and return it.
                    Item::Mod(mod_) => return Self::retrieve_next_mod(mod_),
                    _ => continue,
                },
                None => return None,
            }
        }
    }
}

impl ModuleIterator {
    pub fn retrieve_next_mod(mut mod_: ItemMod) -> Option<ItemMod> {
        if !is_in_line_module(&mod_) {
            return get_module_from_module_name(&mod_.ident)
                .as_mut()
                .flat_map_opt(|(p, s)| s.as_mut()
                    .map_err(|e| { error!("{:?}", e); e })
                    .ok()
                    .flat_map_opt(|f| SynHelper::parse_syn_file(f))
                )
                .map(|f| ItemMod {
                    attrs: f.attrs,
                    vis: Visibility::Inherited,
                    mod_token: Default::default(),
                    ident: mod_.ident.clone(),
                    content: Some((token::Brace::default(), f.items)),
                    semi: Some(token::Semi::default()),
                });
        }

        Some(mod_)
    }

}

fn iter_modules<'a>(ast: Vec<Item>) -> ModuleIterator {
    ModuleIterator { iter: Box::new(ast.into_iter()) }
}

#[test]
fn test_visit() {
    // Parse some example Rust code
    let code = r#"
        mod m1 {
            mod m2 {
                fn bar() {}
            }
            fn foo() {}
        }
    "#;

    let parsed = syn::parse_file(code).unwrap();

    // Iterate over modules in the parsed AST
    for module in iter_modules(parsed.items) {
        println!("Module: {}", module.ident);
        for m in iter_modules(module.content.unwrap().1) {
            println!("Module: {}", m.ident);
        }
    }
}
