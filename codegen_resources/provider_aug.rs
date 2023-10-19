/// TODO: HandlerMapping is going to parse all ApplicationContext in web framework (will be changed
///  to RequestContext) into a map with AntPathMatcher from #[controller(/v1/path/here)], and then each request will be tested to see
///  which AntPathMatcher matches and the request will be completed accordingly. In this way, a new
///  thread won't have to be created for each type of Request and Response generic type argument.
///
///  This will simply be a hook into another project, instead of writing all of it here.
///
/// Use a form of ServiceLoader - you provide code here and then the user provides a path to list of structs or
/// traits or functions. In this way, the user can provide a dependency that will be used dynamically. So
/// the spring.factories will be the path to use statements, and then these use statements will be added
/// at the top of a file like this, and will be pointing to the function that will be used to implement
/// the logic needed to implement... Perhaps it will be a tuple, a use statement and the type of provider...
/// Or a toml file, with the key being the provider type and the value being the path to the function or struct
/// that will be used to implement the logic for the provider.
///
/// This will be generated from the knockoff_factories.toml
#[token_provider]
pub mod handler_mapping_token_provider {

    pub struct HandlerMappingTokenProvider {
        mapping_builder: HandlerMappingBuilder
    }

    impl TokenStreamProvider for HandlerMappingTokenProvider {
        fn new(items: &mut ProfileTree) -> Self {
            Self {
                mapping_builder: HandlerMappingBuilder::new(items)
            }
        }
    }

    impl TokenStreamGenerator for HandlerMappingTokenProvider {
        fn generate_token_stream(&self) -> TokenStream {
            self.mapping_builder.generate_token_stream()
        }
    }

}