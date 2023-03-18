#[controller]
pub struct TestController;

pub struct TestRequestBody;

#[request_mapping(/v1/api/test)]
impl TestController {
    #[get_mapping(/one)]
    pub fn get_test_request_body(#[request_param(test_request_param)] test_request_param: &str) -> String {
        String::default()
    }
}

#[request_mapping(/v1/api/test)]
impl TestController {
    #[get_mapping(/one)]
    pub fn get_test_request_body(#[request_param] test_request_param: &str) -> String {
        String::default()
    }
}
