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

#[derive(Clone, Default, Debug)]
pub struct AntStringRequestMatcher {
    to_match: String,
    splitter: String
}

impl AntStringRequestMatcher {
    pub fn new(to_match: String, splitter: String) -> Self {
        Self {
            to_match, splitter
        }
    }

    fn split_for_match<'a>(&'a self, to_match: &'a str) -> Vec<&str> {
        let split_self_match = to_match.split(self.splitter.as_str())
            .filter(|split| split.len() != 0)
            .collect::<Vec<&str>>();
        split_self_match
    }

    fn do_match(split_match: &Vec<&str>, i: usize, matched: Vec<&str>) -> bool {
        for or_self in matched.iter() {
            let x = split_match.get(i).unwrap();
            let first = or_self.to_string();
            let second = x.to_string();
            if first == second {
                return true;
            }
        }
        false
    }
}

impl Matcher<&str> for AntStringRequestMatcher {
    fn matches(&self, to_match: &str) -> bool {
        let split_self_match = self.split_for_match(&self.to_match);
        let split_match = self.split_for_match(to_match);
        for i in 0..split_match.len() {
            if split_self_match.len() < i + 1 {
                return false;
            }
            let self_to_match = split_self_match.get(i);
            let matched = self_to_match.or(Some(&""))
                .unwrap()
                .split("|")
                .filter(|s| s.len() != 0)
                .collect::<Vec<&str>>();
            if matched.len() > 1 {
                if Self::do_match(&split_match, i, matched) {
                    continue;
                }
            } else {
                if self_to_match == split_match.get(i) {
                    continue;
                }
            }
            return match self_to_match {
                Some(&"**") => {
                    true
                }
                Some(&"*") => {
                    if split_match.len() - i > 1 {
                        continue;
                    }
                    true
                }
                _ => {
                    false
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
    request_matchers: Vec<AntStringRequestMatcher>
}

impl AntPathRequestMatcher {
    pub fn new(request_matchers: Vec<AntStringRequestMatcher> ) -> Self {
        Self {
            request_matchers
        }
    }
}
