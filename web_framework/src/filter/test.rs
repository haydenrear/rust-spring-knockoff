#[cfg(test)]
mod test_filter {

    use crate::filter::filter::{Filter, FilterChain, Controller, MessageConverter, MessageType, JsonMessageConverter, ConverterRegistry, Registration, Converters, Context, ConverterContext, RequestExtractor};
    use crate::request::request::EndpointMetadata;
    use crate::request::request::{HttpRequest, HttpResponse};
    use std::cell::RefCell;
    use std::collections::LinkedList;
    use lazy_static::lazy_static;
    use serde::{Serialize,Deserialize};

    struct TestFilter;

    impl Default for TestFilter {
        fn default() -> Self {
            Self {}
        }
    }

    impl Clone for TestFilter {
        fn clone(&self) -> Self {
            Self {}
        }
    }

    impl Filter for TestFilter {
        fn filter(&self, request: HttpRequest, response: HttpResponse, mut filter: FilterChain) {
            filter.do_filter(request, response);
        }
    }

    pub struct GetControllerImpl {
        pub context: Context
    }

    impl Default for GetControllerImpl {
        fn default() -> Self {
            Self {
                context: Context::default()
            }
        }
    }

    #[derive(Serialize,Deserialize,Debug)]
    pub struct Example {
        value: String
    }

    impl Default for Example {
        fn default() -> Self {
            Example {
                value: String::from("hello!")
            }
        }
    }

    impl <'a> Controller<'a, Example, Example> for GetControllerImpl {
        fn do_request(&self, request: HttpRequest, action: &'a dyn Fn(EndpointMetadata, Example, &Context) -> Example) {
            let converted = self.context.convert(&request);
            let found = match converted {
                Some(s) => {
                    s
                }
                None => {
                    MessageType {
                        message: Example::default()
                    }
                }
            };

            let metadata = self.context.convert_extract(&request);

            action(
                metadata,
                found.message,
                &self.context
            );
        }
    }

    struct TestMessageConverter;

    struct TestFilter2 {
        get: GetControllerImpl
    }

    impl Default for TestFilter2 {
        fn default() -> Self {
            Self {get: GetControllerImpl::default()}
        }
    }

    impl Clone for TestFilter2 {
        fn clone(&self) -> Self {
            Self { get: GetControllerImpl::default() }
        }
    }

    impl Filter for TestFilter2 {
        fn filter(&self, request: HttpRequest, response: HttpResponse, mut filter: FilterChain) {
            self.get.do_request(request.clone(), &|m, r, c| {
                println!("hello world!");
                r
            });
            filter.do_filter(request, response);
        }
    }

    #[derive(Serialize, Deserialize)]
    struct TestJson {
        value: String
    }


    #[test]
    fn test_filter() {
        let filter = TestFilter::default();
        let one: &dyn Filter = &filter;
        let mut fc = FilterChain::new(vec![one]);
        fc.do_filter(HttpRequest::default(), HttpResponse::default());
        assert_eq!(fc.num, 0);
    }


    #[test]
    fn test_get_in_filter() {
        let filter = TestFilter2::default();
        let one: &dyn Filter = &filter;
        let mut fc = FilterChain::new(vec![one]);
        let mut request = HttpRequest::default();
        request.body = serde_json::to_string(&Example::default())
            .unwrap();
        fc.do_filter(request, HttpResponse::default());
        assert_eq!(fc.num, 0);
    }

}