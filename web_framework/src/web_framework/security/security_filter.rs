pub mod security_filter {
    use crate::web_framework::context::ApplicationContext;
    use crate::web_framework::filter::filter::{Filter, FilterChain};
    use crate::web_framework::request::request::{WebRequest, WebResponse};

    pub struct SecurityFilter;

    impl Filter for SecurityFilter {
        fn filter(&self, request: &WebRequest, response: &mut WebResponse, ctx: &ApplicationContext) {
            todo!()
        }

    }

}