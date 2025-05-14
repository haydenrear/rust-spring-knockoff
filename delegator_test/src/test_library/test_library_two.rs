pub mod test_library_six;

use std::fmt::{Debug, Formatter};
use spring_knockoff_boot_macro::{autowired, bean, enable_http_security, service, qualifier, message_converter};
use web_framework::web_framework::security::http_security::HttpSecurity;
use crate::test_library::test_library_four::One;
use serde::{Deserialize, Serialize};
use web_framework::web_framework::convert::MessageConverter;
use web_framework::web_framework::message::MessageType;
use web_framework_shared::WebRequest;

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

#[service(TestMessageConverter)]
#[derive(Default)]
#[message_converter]
pub struct TestMessageConverter {
    
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct AnotherRequestTwo{
    pub value: String
}

#[message_converter(media_type = "application/json")]
impl MessageConverter<AnotherRequestTwo, AnotherRequestTwo> for TestMessageConverter {
    fn new_message_converter() -> Self
    where
        Self: Sized
    {
        TestMessageConverter {}
    }

    fn convert_to(&self, request: &WebRequest) -> Option<MessageType<AnotherRequestTwo>> {
        None
    }

    fn convert_from(&self, request_body: &AnotherRequestTwo, request: &WebRequest) -> Option<String> {
        None
    }

    fn do_convert(&self, request: &WebRequest) -> bool {
        false
    }

    fn message_type(&self) -> Vec<String> {
        vec![String::from("application/json")]
    }
}