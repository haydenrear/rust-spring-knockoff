pub mod request {
    use crate::session::session::HttpSession;
    use std::collections::HashMap;

    trait HttpEntity {}

    pub struct HttpRequest {
        pub session: HttpSession,
        pub headers: HashMap<String, String>,
        pub body: String,
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
