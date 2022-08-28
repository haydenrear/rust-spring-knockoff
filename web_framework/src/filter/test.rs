#[cfg(test)]
mod test_filter {
    use crate::filter::filter::{Filter, FilterChain};
    use crate::request::request::{HttpRequest, HttpResponse};
    use std::cell::RefCell;

    struct TestFilter {}

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

    #[test]
    fn test_filter() {
        let filter = TestFilter::default();
        let one: Box<&dyn Filter> = Box::new(&filter);
        let mut fc = FilterChain::new(vec![one]);
        fc.do_filter(HttpRequest::default(), HttpResponse::default());
        assert_eq!(fc.num, 0);
    }
}
