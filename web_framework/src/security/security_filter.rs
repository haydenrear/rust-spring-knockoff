pub mod security_filter {
    use crate::context::ApplicationContext;
    use crate::filter::filter::{Filter, FilterChain};
    use crate::request::request::{WebRequest, WebResponse};

    pub struct SecurityFilter;

    impl Filter for SecurityFilter {
        fn filter(&self, request: &WebRequest, response: &mut WebResponse, ctx: &ApplicationContext) {
            todo!()
        }

        fn dyn_clone(&self) -> Box<dyn Filter> {
            todo!()
        }
    }

}