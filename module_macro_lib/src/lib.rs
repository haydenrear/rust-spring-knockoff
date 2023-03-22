#![feature(pattern)]

include!(concat!(env!("OUT_DIR"), "/codegen.rs"));

pub mod module_macro_lib {
    pub mod parse_container;
    pub mod module_parser;
    pub mod knockoff_context_builder;
    pub mod profile_tree;
    pub mod util;
    pub mod bean_parser;
    pub mod context_builder;
    pub mod knockoff_context;
    pub mod debug;
    pub mod logging;
    pub mod item_parser;
    pub mod item_modifier;
    #[cfg(test)]
    pub mod test;
}
