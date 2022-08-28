pub mod test;
pub mod filter {

    extern crate alloc;
    extern crate core;

    use crate::request::request::{HttpRequest, HttpResponse};
    use crate::session::session::HttpSession;
    use alloc::string::String;
    use core::borrow::{Borrow, BorrowMut};
    use serde::{Deserialize, Serialize};
    use std::collections::{HashMap, LinkedList};

    #[derive(Clone)]
    pub struct FilterChain<'a> {
        filters: Vec<Box<&'a dyn Filter>>,
        pub(crate) num: usize,
    }

    impl<'a> FilterChain<'a> {
        pub fn do_filter(&mut self, request: HttpRequest, response: HttpResponse) {
            let next = self.next();
            if next != -1 {
                let f = &self.filters[(next - 1) as usize];
                f.filter(request, response, self.clone());
                if self.num >= self.filters.len() {
                    self.num = 0;
                }
            }
        }

        pub(crate) fn next(&mut self) -> i64 {
            if self.filters.len() > self.num {
                self.num += 1;
                return self.num as i64;
            } else {
                -1
            }
        }

        pub fn new(filters: Vec<Box<&'a dyn Filter>>) -> Self {
            Self {
                filters: filters,
                num: 0,
            }
        }
    }

    pub trait Filter {
        fn filter(&self, request: HttpRequest, response: HttpResponse, filter: FilterChain);
    }
}
