pub mod request {

    use crate::session::session::HttpSession;
    use std::collections::HashMap;
    use std::fs::Metadata;
    use std::net::TcpStream;
    use std::ops::Deref;
    use crate::context::Context;
    use crate::http::HttpMethod;

    #[derive(Clone)]
    pub struct EndpointMetadata {
        pub path_variables: String,
        pub query_params: String
    }

    impl Default for EndpointMetadata {
        fn default() -> Self {
            Self {
                path_variables: String::default(),
                query_params: String::default()
            }
        }
    }

    trait HttpEntity {}

    pub struct HttpRequest {
        pub headers: HashMap<String, String>,
        pub body: String,
        pub metadata: EndpointMetadata,
        pub method: HttpMethod,
    }

    impl Clone for HttpRequest {
        fn clone(&self) -> Self {
            Self {
                headers: self.headers.clone(),
                body: self.body.clone(),
                metadata: self.metadata.clone(),
                method: HttpMethod::Get
            }
        }
    }

    impl Clone for HttpMethod {
        fn clone(&self) -> Self {
            match self {
                HttpMethod::Post => {
                    HttpMethod::Post
                },
                HttpMethod::Get => {
                    HttpMethod::Get
                }
            }
        }
    }

    pub trait RequestExtractor<T> {
        fn convert_extract(&self, request: &HttpRequest) -> Option<T>;
    }

    impl RequestExtractor<EndpointMetadata> for Context {
        fn convert_extract(&self, request: &HttpRequest) -> Option<EndpointMetadata> {
            Some(EndpointMetadata::default())
        }
    }

    #[derive(Clone)]
    pub struct HttpResponse {
        pub session: HttpSession,
        pub response: String
    }

    pub trait ResponseWriter {
        fn write(&mut self, response: &[u8]);
    }

    impl ResponseWriter for HttpResponse {
        fn write(&mut self, response: &[u8]) {
            self.response = String::from_utf8(Vec::from(response)).map(| response_str| {
                self.response.clone() + response_str.as_str()
            }).unwrap_or(self.response.clone());
        }
    }

    impl Default for HttpRequest {
        fn default() -> Self {
            Self {
                headers: HashMap::new(),
                body: String::default(),
                metadata: EndpointMetadata::default(),
                method: HttpMethod::Get
            }
        }
    }

    impl Default for HttpResponse {
        fn default() -> Self {
            Self {
                session: HttpSession::default(),
                response: String::default()
            }
        }
    }

}
