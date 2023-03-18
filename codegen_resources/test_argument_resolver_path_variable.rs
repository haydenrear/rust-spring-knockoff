#[controller]
pub struct TestController;

pub struct TestRequestBody;

#[request_mapping(/v1/api/test)]
impl TestController {
    #[get_mapping(/one/{two})]
    pub fn get_test_request_body(#[path_variable(two)] two: &str) -> String {
        String::default()
    }
}

#[request_mapping(/v1/api/test)]
impl TestController {
    #[get_mapping(/one/{two})]
    pub fn get_test_request_body(#[path_variable] two: &str) -> String {
        String::default()
    }
}
