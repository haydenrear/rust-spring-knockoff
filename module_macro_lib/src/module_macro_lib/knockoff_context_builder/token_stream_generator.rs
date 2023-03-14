use proc_macro2::TokenStream;
use syn::parse2;
use syn::token::Use;
use module_macro_codegen::parser::CodegenItem;
use module_macro_shared::profile_tree::ProfileTree;
use knockoff_providers_gen::HandlerMappingTokenProvider;

pub trait TokenStreamGenerator {
    fn generate_token_stream(&self) -> TokenStream;
}

pub struct UserProvidedTokenStreamGenerator {
    handler_mapping_token_provider: HandlerMappingTokenProvider
}

impl UserProvidedTokenStreamGenerator {
    pub(crate) fn new(profile_tree: &ProfileTree) -> Self {
        let handler_mapping_token_provider = HandlerMappingTokenProvider::new(profile_tree);
        Self {
            handler_mapping_token_provider
        }
    }
}

impl TokenStreamGenerator for UserProvidedTokenStreamGenerator {
    fn generate_token_stream(&self) -> TokenStream {
        // self.handler_mapping_token_provider.generate_token_stream()
        self.handler_mapping_token_provider.generate_token_stream()
    }
}