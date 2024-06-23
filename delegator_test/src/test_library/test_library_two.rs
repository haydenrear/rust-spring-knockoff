pub mod test_library_six;

use std::fmt::{Debug, Formatter};
use spring_knockoff_boot_macro::{autowired, bean, enable_http_security, service, qualifier};
use web_framework::web_framework::security::http_security::HttpSecurity;
use crate::test_library::test_library_four::One;
use serde::{Deserialize, Serialize};

#[derive(Default, Debug)]
#[service(Once)]
pub struct Ten {}

#[enable_http_security]
pub fn enable_http_security<Request, Response>(http: &mut HttpSecurity<Request, Response>)
    where
        Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
        Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static
{
    http.request_matcher(vec!["one", "two"], vec!["authority_one"]);
    // http.authentication_provider(Box::new());
}


#[service(SevenIntercept)]
#[derive(Default)]
pub struct SevenIntercept {
    pub two: String,
}


impl SevenIntercept {
    pub fn one_two_three(&self, one: SevenIntercept) -> String {
        print!("testing...");
        print!("{} is one", one.two.to_string());
        "two one".to_string()
    }
}

