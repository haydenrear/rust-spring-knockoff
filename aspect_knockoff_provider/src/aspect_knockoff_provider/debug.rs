use std::fmt;
use std::fmt::{Debug, Formatter};
use codegen_utils::syn_helper;
use crate::aspect_knockoff_provider::{AspectInfo, MethodAdviceChain};
use crate::aspect_knockoff_provider::aspect_ts_generator::AspectGenerator;

impl Debug for MethodAdviceChain {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut debug_struct = f.debug_struct("MethoAdviceChain");
        syn_helper::debug_struct_field_opt_tokens(&mut debug_struct, &self.before_advice, "before_advice");
        syn_helper::debug_struct_field_opt_tokens(&mut debug_struct, &self.after_advice, "after_advice");
        syn_helper::debug_struct_field_opt_tokens(&mut debug_struct, &self.proceed_statement, "proceed_statement");
        debug_struct.finish()
    }
}

impl Debug for AspectInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut debug_struct = f.debug_struct("AspectInfo");
        syn_helper::debug_struct_vec_field_debug("advice chain", &mut debug_struct, &self.advice_chain);
        syn_helper::debug_struct_field_opt_tokens(&mut debug_struct, &self.return_type, "return_type");
        syn_helper::debug_struct_field_opt_tokens(&mut debug_struct, &self.method, "method_before");
        syn_helper::debug_struct_field_opt_tokens(&mut debug_struct, &self.method_after, "method_after");
        syn_helper::debug_struct_field_opt_tokens(&mut debug_struct, &self.original_fn_logic, "original_fn_logic");
        debug_struct.field("method_advice_aspect", &self.method_advice_aspect);
        debug_struct.field("mutable", &self.mutable.to_string().as_str());
        debug_struct.field("args", &self.args.iter().map(|a| {
            let mut type_and_ident = "Ident: ".to_string();
            type_and_ident +=  a.0.to_string().as_str();
            type_and_ident += "Type: ";
            type_and_ident += a.0.to_string().as_str();
            type_and_ident
        }).collect::<Vec<String>>().join(", ").as_str());
        debug_struct.finish()
    }
}

impl Debug for AspectGenerator {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let _ = f.debug_struct("AspectGenerator");
        f.debug_list()
            .entries(&self.method_advice_aspects);
        Ok(())
    }
}

