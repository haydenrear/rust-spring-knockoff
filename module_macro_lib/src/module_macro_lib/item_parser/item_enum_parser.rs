use syn::{Fields, ItemEnum};
use codegen_utils::syn_helper::SynHelper;
use crate::module_macro_lib::bean_parser::{BeanDependencyParser};
use crate::module_macro_lib::item_parser::{get_profiles, ItemParser};
use module_macro_shared::bean::BeanDefinition;
use module_macro_shared::parse_container::ParseContainer;
use quote::ToTokens;

pub struct ItemEnumParser;

use knockoff_logging::{initialize_log, use_logging};
use_logging!();
initialize_log!();
use crate::module_macro_lib::logging::executor;
use crate::module_macro_lib::logging::StandardLoggingFacade;

//TODO: the fields here may screw things up. Enum is not ready to be autowired...
impl ItemParser<ItemEnum> for ItemEnumParser {
    fn parse_item(parse_container: &mut ParseContainer, enum_to_add: &mut ItemEnum, path_depth: Vec<String>) {
        log_message!("adding type with name {}", enum_to_add.ident.clone().to_token_stream().to_string());
        &mut parse_container.injectable_types_builder.get_mut(&enum_to_add.ident.to_string().clone())
            .map(|struct_impl: &mut BeanDefinition| {
                struct_impl.enum_found = Some(enum_to_add.clone());
            })
            .or_else(|| {
                let enum_fields = enum_to_add.variants.iter()
                    .map(|variant| variant.fields.clone())
                    .collect::<Vec<Fields>>();

                let mut impl_found = BeanDefinition {
                    struct_type: None,
                    path_depth,
                    struct_found: None,
                    traits_impl: vec![],
                    enum_found: Some(enum_to_add.clone()),
                    deps_map: vec![],
                    id: enum_to_add.ident.clone().to_string(),
                    profile: get_profiles(&enum_to_add.attrs),
                    ident: Some(enum_to_add.ident.clone()),
                    fields: enum_fields,
                    bean_type: BeanDependencyParser::get_bean_type_opt(&enum_to_add.attrs),
                    mutable: SynHelper::get_attr_from_vec(&enum_to_add.attrs, &vec!["mutable_bean"])
                        .map(|_| true)
                        .or(Some(false)).unwrap(),
                    aspect_info: vec![],
                    factory_fn: None,
                    metadata: "".to_string(),
                };
                parse_container.injectable_types_builder.insert(enum_to_add.ident.to_string().clone(), impl_found);
                None
            });
    }
}
