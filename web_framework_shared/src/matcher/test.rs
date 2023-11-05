use std::collections::LinkedList;
use std::str::FromStr;
use http::Uri;
use crate::matcher::{AntPathRequestMatcher, AntStringRequestMatcher, Matcher};
use crate::request::WebRequest;

#[test]
fn test_ant_path_request_matcher() {
    let first = create_request_matcher("/v1/test_one/**".to_string(), "/".to_string());
    let second = create_request_matcher("/v1/test_two/*".to_string(), "/".to_string());
    let third = create_request_matcher("/v1/test_three".to_string(), "/".to_string());
    let fourth = create_request_matcher("/v1/test_one_hundred|test_four_hundred/one/two/**".to_string(), "/".to_string());

    let mut request_matchers = vec![];

    request_matchers.push(first);
    request_matchers.push(second);
    request_matchers.push(third);
    request_matchers.push(fourth);

    let request_matcher = create_request_matchers(request_matchers);

    assert!(request_matcher.matches(&test_web_request("/v1/test_one/okay/two/three".to_string())));
    assert!(request_matcher.matches(&test_web_request("/v1/test_one".to_string())));
    assert!(!request_matcher.matches(&test_web_request("/v1/test_two".to_string())));
    assert!(request_matcher.matches(&test_web_request("/v1/test_two/okay".to_string())));
    assert!(request_matcher.matches(&test_web_request("/v1/test_two/two".to_string())));
    assert!(request_matcher.matches(&test_web_request("/v1/test_one_hundred/one/two/three_four".to_string())));
    assert!(request_matcher.matches(&test_web_request("/v1/test_four_hundred/one/two/three_four".to_string())));
    assert!(request_matcher.matches(&test_web_request("/v1/test_four_hundred/one/two".to_string())));

    assert!(!request_matcher.matches(&test_web_request("/v1/test_two/okay/two/three".to_string())));
    assert!(!request_matcher.matches(&test_web_request("/v1/test_three/okay/two/three".to_string())));

}

#[test]
fn test_ant_path_matcher() {
    let second = create_request_matcher("/v1/test_one/*/one".to_string(), "/".to_string());
    assert!(second.matches("/v1/test_one/two/one"));
    assert!(second.matches("/v1/test_one/three/one"));
    assert!(!second.matches("/v1/test_one/three/two"));
    assert!(!second.matches("/v1/test_one/two"));
}

#[test]
fn test_request_matcher() {
    let first = create_request_matcher("o*e".to_string(), "".to_string());
    assert!(first.matches("one"));
}

fn create_request_matcher(to_match: String, splitter: String) -> AntStringRequestMatcher {
    return AntStringRequestMatcher::new(to_match, splitter)
}

fn create_request_matchers(request_matchers: Vec<AntStringRequestMatcher>) -> AntPathRequestMatcher {
    AntPathRequestMatcher::new_from_request_matcher(request_matchers)
}

pub fn test_web_request(to_match: String) -> WebRequest {
    let mut wr = WebRequest::default();
    let x = to_match.as_str();
    let string = format!("https://test{}", x);
    wr.uri = Uri::from_str(&string).unwrap();
    wr
}

