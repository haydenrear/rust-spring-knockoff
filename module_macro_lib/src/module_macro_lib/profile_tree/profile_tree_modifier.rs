use std::collections::HashMap;
use quote::{quote, ToTokens};
use syn::{parse2, Type};
use codegen_utils::syn_helper::SynHelper;
use module_macro_shared::bean::BeanDefinition;
use knockoff_logging::{initialize_log, use_logging};
use_logging!();
initialize_log!();

use crate::module_macro_lib::logging::executor;
use crate::module_macro_lib::logging::StandardLoggingFacade;
use module_macro_shared::profile_tree::ProfileTree;
