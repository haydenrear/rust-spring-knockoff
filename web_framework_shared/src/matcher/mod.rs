use std::collections::LinkedList;
use derive_syn_parse::Parse;
use proc_macro2::TokenStream;
use syn::__private::quote::quote;
use syn::__private::ToTokens;
use crate::request::WebRequest;

use knockoff_logging::*;
use lazy_static::lazy_static;
use std::sync::Mutex;
use codegen_utils::project_directory;
use crate::logger_lazy;
import_logger!("matcher.rs");

#[cfg(test)]
pub mod test;

pub trait Matcher<T> {
    fn matches(&self, to_match: T) -> bool;
}

pub trait StringMatcher<'a>: Matcher<&'a str> {
}

pub trait RequestMatcher: for<'a> Matcher<&'a WebRequest> {
}

#[derive(Clone, Default, Debug)]
pub struct AntStringRequestMatcher {
    pub to_match: String,
    pub splitter: String,
    /// if path like /one/two/*, should /one/two count?
    pub count_last_with_single_star: bool,
    /// if path like /one/two/**, should /one/two count?
    pub count_last_with_double_star: bool
}

impl AntStringRequestMatcher {
    pub fn new(to_match: String, splitter: String) -> Self {
        Self {
            to_match: to_match.to_string(), splitter,
            count_last_with_single_star: false,
            count_last_with_double_star: true,
        }
    }

    fn split_for_match<'a>(&'a self, to_match: &'a str) -> Vec<&str> {
        let split_self_match = to_match.split(self.splitter.as_str())
            .filter(|split| split.len() != 0)
            .collect::<Vec<&str>>();
        split_self_match
    }

    fn do_match(to_match_values: &Vec<&str>, i: usize, matcher_values: Vec<&str>) -> bool {
        for or_self in matcher_values.iter() {
            let to_match_val = to_match_values.get(i).unwrap();
            return Self::match_value(to_match_val, or_self);
        }
        false
    }

    fn match_value(to_match: &str, matcher: &str) -> bool {
        if matcher.contains("*") && matcher != "*" {
            /// Cover cases where glob is in middle of word.
            let matcher = AntStringRequestMatcher::new(matcher.to_string(), "".to_string()) ;
            matcher.matches(to_match)
        } else if to_match == matcher {
            true
        } else {
            false
        }
    }

    fn is_last_star(&self, split_self_match: Vec<&str>, split_self_has_one_added: bool) -> bool {
        let is_last_single_star = split_self_has_one_added && *split_self_match.last().unwrap() == "**" && self.count_last_with_double_star;
        let is_last_double_star = split_self_has_one_added && *split_self_match.last().unwrap() == "*" && self.count_last_with_single_star;
        is_last_single_star || is_last_double_star
    }
}

impl Matcher<&str> for AntStringRequestMatcher {
    fn matches(&self, to_match: &str) -> bool {

        let split_self_match = self.split_for_match(&self.to_match);
        let to_match_value = self.split_for_match(to_match);

        for i in 0..to_match_value.len() {

            if i > split_self_match.len() {
                return false;
            }

            let self_to_match = split_self_match.get(i);

            match self_to_match {
                Some(&"**") => {
                    return true
                }
                Some(&"*") => {
                    if i == to_match_value.len() - 1 {
                        return split_self_match.len() == to_match_value.len();
                    }
                    continue;
                }
                _ => {}
            }

            let matcher_value = self_to_match.or(Some(&""))
                .unwrap()
                .split("|")
                .filter(|s| s.len() != 0)
                .collect::<Vec<&str>>();

            if matcher_value.len() > 1 {
                if Self::do_match(&to_match_value, i, matcher_value) {
                    continue;
                }
            } else {
                let to_match_opt = to_match_value.get(i);
                if to_match_opt.as_ref().is_some() && self_to_match.as_ref().is_some()
                    && Self::match_value(
                    to_match_opt.as_ref().unwrap().to_string().to_string().as_str(),
                    self_to_match.as_ref().unwrap().to_string().as_str()) {
                    continue;
                } else {
                    return false;
                }
            }
        }

        if split_self_match.len() > to_match_value.len() {
            let split_self_has_one_added = split_self_match.len() - to_match_value.len() == 1;
            if self.is_last_star(split_self_match, split_self_has_one_added) {
                true
            } else {
               false
            }
        } else {
            if split_self_match.len() == to_match_value.len() {
                split_self_match.last() == to_match_value.last()
            } else {
                false
            }
        }


    }

}

impl StringMatcher<'_> for AntStringRequestMatcher {
}

impl Matcher<&'_ WebRequest> for AntPathRequestMatcher {
    fn matches(&self, to_match: &WebRequest) -> bool {
        self.request_matchers.iter()
            .any(|r| r.matches(to_match.uri.path()))
    }
}

#[derive(Clone)]
pub struct AntPathRequestMatcher {
    //TODO: add bloom filter and contains
    pub request_matchers: Vec<AntStringRequestMatcher>
}

impl AntPathRequestMatcher {
    pub fn new(to_match: &str, splitter: &str) -> Self {
        let request_matchers = to_match.split(splitter).map(|to_match|
            AntStringRequestMatcher::new(
                to_match.to_string(),
                " ".to_string(),
            ))
            .collect::<Vec<AntStringRequestMatcher>>();
        Self {
            request_matchers
        }
    }
    pub fn new_from_request_matcher(request_matchers: Vec<AntStringRequestMatcher> ) -> Self {
        Self {
            request_matchers
        }
    }
}
