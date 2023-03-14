use syn::{Block, ImplItemMethod, Stmt, Type};
use module_macro_codegen::aspect::MethodAdviceAspectCodegen;
use proc_macro2::Ident;

use knockoff_logging::{initialize_log, use_logging};
use_logging!();
initialize_log!();
use crate::logging::executor;
use crate::logging::StandardLoggingFacade;

#[derive(Default, Clone)]
pub struct MethodAdviceChain {
    pub before_advice: Option<Block>,
    pub after_advice: Option<Block>,
    pub proceed_statement: Option<Stmt>
}

impl MethodAdviceChain {
    pub fn new(method_advice: &MethodAdviceAspectCodegen) -> Self {
        Self {
            before_advice: method_advice.before_advice.clone(),
            after_advice: method_advice.after_advice.clone(),
            proceed_statement: method_advice.proceed_statement.clone(),
        }
    }
}

#[derive(Default, Clone)]
pub struct AspectInfo {
    pub method_advice_aspect: MethodAdviceAspectCodegen,
    pub method: Option<ImplItemMethod>,
    pub args: Vec<(Ident, Type)>,
    /// This is the block before any aspects are added.
    pub original_fn_logic: Option<Block>,
    pub return_type: Option<Type>,
    pub method_after: Option<ImplItemMethod>,
    pub mutable: bool,
    pub advice_chain: Vec<MethodAdviceChain>
}
