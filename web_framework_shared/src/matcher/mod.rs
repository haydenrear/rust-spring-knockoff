use std::collections::LinkedList;
use crate::request::WebRequest;

#[cfg(test)]
pub mod test;

pub trait Matcher<T> {
    fn matches(&self, to_match: T) -> bool;
}

pub trait StringMatcher<'a>: Matcher<&'a str> {
}

pub trait RequestMatcher: Matcher<WebRequest> {
}

pub struct AntStringRequestMatcher {
    to_match: String
}

impl AntStringRequestMatcher {
    pub fn new(to_match: String) -> Self {
        Self {
            to_match
        }
    }

    fn split_for_match(to_match: &str) -> Vec<&str> {
        let split_self_match = to_match.split("/").collect::<Vec<&str>>();
        split_self_match
    }
}

impl Matcher<&str> for AntStringRequestMatcher {
    fn matches(&self, to_match: &str) -> bool {
        let split_self_match = Self::split_for_match(&self.to_match);
        let split_match = Self::split_for_match(to_match);
        for i in 0..split_match.len() {
            let self_to_match = split_self_match.get(i);
            if self_to_match == split_match.get(i) {
                continue
            } else {
                return match self_to_match {
                    Some(&"**") => {
                        true
                    }
                    Some(&"*") => {
                        if split_match.len() - i > 1 {
                            return false;
                        }
                        true
                    }
                    _ => {
                        false
                    }
                }
            }
        }
        true
    }
}

impl StringMatcher<'_> for AntStringRequestMatcher {
}

impl Matcher<WebRequest> for AntPathRequestMatcher {
    fn matches(&self, to_match: WebRequest) -> bool {
        self.request_matchers.iter()
            .any(|r| r.matches(&to_match.metadata.base_uri))
    }
}

pub struct AntPathRequestMatcher {
    //TODO: add bloom filter and contains
    request_matchers: LinkedList<AntStringRequestMatcher>
}

impl AntPathRequestMatcher {
    pub fn new(request_matchers: LinkedList<AntStringRequestMatcher> ) -> Self {
        Self {
            request_matchers
        }
    }
}
