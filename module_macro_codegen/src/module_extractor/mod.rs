use std::collections::LinkedList;
use std::ops::Deref;
use quote::{quote, ToTokens};
use syn::Item;
use knockoff_logging::{initialize_log, use_logging};
use crate::codegen_items;
use crate::parser::{CodegenItem, CodegenItems, get_codegen_item};

use_logging!();
initialize_log!();

use crate::logger::executor;
use crate::logger::StandardLoggingFacade;

#[derive(Default)]
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

    pub(crate) fn new(item: &Item) -> Option<Box<dyn CodegenItem>> {
        if ModuleParser::supports_item(item) {
            return Self::get_codegen_items(item)
                .map(|c| Some(Box::new(Self{codegen_items: c }) as Box<dyn CodegenItem>))
                .flatten()
                .or(None);
        }
        None
    }

    pub(crate) fn get_codegen_items(tokens: &Item) -> Option<CodegenItems> {
        match tokens {
            Item::Mod(module_to_parse) => {
                log_message!("{} is name of a configuration module found.", module_to_parse.ident.to_token_stream().to_string().as_str());
                Some(
                    CodegenItems{
                        codegen: module_to_parse.content.iter()
                                    .flat_map(|inner_item| inner_item.1.iter())
                                    .map(|inner_item| get_codegen_item(inner_item))
                                    .flatten()
                                    .collect()
                    }
                )
            }
            _ => {
                None
            }
        }
    }
}

impl CodegenItem for ModuleParser {

    fn supports_item(item: &Item) -> bool {
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
    }

    fn supports(&self, item: &Item) -> bool {
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

    fn clone_dyn_codegen(&self) -> Box<dyn CodegenItem> {
        Box::new(ModuleParser::default())
    }

    fn get_unique_id(&self) -> String {
        self.get_unique_ids()
            .join(", ")
    }

    fn get_unique_ids(&self) -> Vec<String> {
        self.codegen_items.codegen.iter()
            .map(|c| c.get_unique_id())
            .collect()
    }
}