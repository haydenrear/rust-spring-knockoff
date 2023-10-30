use std::any::Any;
use std::collections::LinkedList;
use std::ops::Deref;
use std::os::unix::raw::time_t;
use quote::{quote, ToTokens};
use rand::distributions::Alphanumeric;
use rand::Rng;
use syn::Item;
use crate::codegen_items;
use crate::parser::{CodegenItem, CodegenItems, CodegenItemType, get_codegen_item, LibParser};

use knockoff_logging::*;
use lazy_static::lazy_static;
use std::sync::Mutex;
use codegen_utils::project_directory;
use crate::logger_lazy;
import_logger!("module_extractor.rs");

#[derive(Default, Clone)]
pub struct ModuleParser {
    codegen_items: CodegenItems,
}

/// Basic idea is that any module marked with #[configuration] will be eligible to be parsed, and then
/// based on the annotation it is parsed and created. Then, aggregating delegator objects will be created
/// using the iterator pattern. For example, any configuration could contain a field aug. Therefore, a
/// delegating field aug is created that delegates to each of those provided by the user, which calls
/// each of those provided.
/// 1. Extract module into tree, into each of the possible types provided by the API.
/// 2. Gather like types and create delegators.
/// Potentially some types provided by the API should only be applied to some modules. This means that
/// each of the created delegators will have a profile passed in so as to only apply to those modules
/// specified by the user.
impl ModuleParser {

    pub(crate) fn new(item: &Vec<Item>) -> Option<Self> {
        if ModuleParser::supports_item(item) {
            return Self::get_codegen_items(item)
                .map(|c| Some(Self{codegen_items: c }))
                .flatten()
                .or(None);
        }
        None
    }

    pub(crate) fn new_dyn_codegen(item: &Vec<Item>) -> Option<CodegenItemType> {
        Self::new(item)
            .map(|i| CodegenItemType::Module(i))
    }

    pub(crate) fn get_codegen_items(tokens: &Vec<Item>) -> Option<CodegenItems> {
        if tokens.len() == 0 {
            return None;
        }
        let codegen = tokens.iter().flat_map(|tokens| {
            match tokens {
                Item::Mod(module_to_parse) => {
                    if module_to_parse.content.is_some() {
                        log_message!("{} is name of a configuration module found.", module_to_parse.ident.to_token_stream().to_string().as_str());
                        return LibParser::gen_codegen_items()
                            .codegen.iter()
                            .filter(|c| Self::supports_item(&module_to_parse.content.clone().unwrap().1))
                            .map(|c| c.clone())
                            .collect::<Vec<CodegenItemType>>();
                    }
                    vec![]
                }
                _ => {
                    vec![]
                }
            }
        })
            .collect::<Vec<CodegenItemType>>();

        Some(CodegenItems{
            codegen
        })

    }
}

impl CodegenItem for ModuleParser {

    fn supports_item(item: &Vec<Item>) -> bool {
        item.iter().any(|item| {
            match item {
                Item::Mod(mod_found) => {
                    log_message!("{} is name of a codegen configuration module.", mod_found.ident.to_token_stream().to_string().as_str());
                    mod_found.attrs.iter()
                        .any(|attr| attr.path.to_token_stream()
                            .to_string().as_str()
                            .contains("configuration")
                        )
                }
                _ => {
                    false
                }
            }
        })
    }

    fn supports(&self, item: &Vec<Item>) -> bool {
        Self::supports_item(item)
    }

    fn get_codegen(&self) -> Option<String> {
        self.codegen_items.codegen.iter().for_each(|codegen_item| {
            log_message!("{} is name of a codegen item in configuration module.", codegen_item.get_unique_id().as_str());
        });
        if self.codegen_items.codegen.len() <= 0 {
            return None;
        }
        Some(
            self.codegen_items
                .codegen
                .iter()
                .map(|codegen_item| codegen_item.get_codegen())
                .flatten()
                .collect::<Vec<String>>()
                .join("")
        )
    }

    fn default_codegen(&self) -> String {
        let ts = quote!{
        };
        ts.to_string()
    }

    fn get_unique_id(&self) -> String {
        rand::thread_rng()
            .sample_iter(&Alphanumeric{})
            .take(10)
            .map(char::from)
            .collect()
    }

    fn get_unique_ids(&self) -> Vec<String> {
        vec![self.get_unique_id()]
    }
}