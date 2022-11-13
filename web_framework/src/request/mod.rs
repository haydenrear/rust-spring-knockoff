pub mod request {
    use crate::session::session::HttpSession;
    use std::collections::HashMap;
    use std::fs::Metadata;

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
        pub session: HttpSession,
        pub headers: HashMap<String, String>,
        pub body: String,
        pub metadata: EndpointMetadata
    }

    impl Clone for HttpRequest {
        fn clone(&self) -> Self {
            Self {
                session: HttpSession::default(),
                headers: HashMap::new(),
                body: String::default(),
                metadata: EndpointMetadata::default()
            }
        }
    }


    pub struct HttpResponse {
        pub session: HttpSession,
    }

    impl Default for HttpRequest {
        fn default() -> Self {
            Self {
                session: HttpSession::default(),
                headers: HashMap::new(),
                body: String::default(),
                metadata: EndpointMetadata::default()
            }
        }
    }

    impl Default for HttpResponse {
        fn default() -> Self {
            Self {
                session: HttpSession::default(),
            }
        }
    }
}
