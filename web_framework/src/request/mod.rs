pub mod request {

    use crate::context::RequestContext;
    use crate::http::{Connection, HttpMethod};
    use crate::session::session::HttpSession;
    use std::collections::{HashMap, LinkedList};
    use std::fs::Metadata;
    use std::io::{Bytes, Read, Write};
    use std::marker::PhantomData;
    use std::net::TcpStream;
    use std::ops::Deref;
    use async_std::io::ReadExt;
    use serde::{Deserialize, Serialize};
    use circular::Buffer;

    #[derive(Clone, Serialize, Deserialize)]
    pub struct EndpointMetadata {
        pub path_variables: String,
        pub query_params: String,
    }

    impl Default for EndpointMetadata {
        fn default() -> Self {
            Self {
                path_variables: String::default(),
                query_params: String::default(),
            }
        }
    }

    trait HttpEntity {}


    #[derive(Serialize, Deserialize)]
    pub struct HttpRequest {
        pub headers: HashMap<String, String>,
        pub body: String,
        pub metadata: EndpointMetadata,
        pub method: HttpMethod,
        // #[serde(skip_serializing, skip_deserializing)]
        // pub connection: Option<&'a dyn Connection<'a, &'a [u8]>>
    }

    impl <'a> HttpRequest {
        // pub fn write(&self, cxn: &dyn Connection<&[u8]>) {
        //     // cxn.write(self.response)
        // }
    }

    impl Clone for HttpRequest {
        fn clone(&self) -> Self {
            Self {
                headers: self.headers.clone(),
                body: self.body.clone(),
                metadata: self.metadata.clone(),
                method: HttpMethod::Get,
                // connection: None,
            }
        }
    }

    impl Clone for HttpMethod {
        fn clone(&self) -> Self {
            match self {
                HttpMethod::Post => HttpMethod::Post,
                HttpMethod::Get => HttpMethod::Get,
            }
        }
    }

    pub trait RequestExtractor<T> {
        fn convert_extract(&self, request: &HttpRequest) -> Option<T>;
    }

    impl RequestExtractor<EndpointMetadata> for RequestContext {
        fn convert_extract(&self, request: &HttpRequest) -> Option<EndpointMetadata> {
            Some(EndpointMetadata::default())
        }
    }

    #[derive(Clone, Serialize, Deserialize)]
    pub struct HttpResponse {
        pub session: HttpSession,
        pub response: String,
        #[serde(skip_serializing, skip_deserializing)]
        pub response_bytes: ResponseBytesBuffer
    }

    #[derive(Clone)]
    pub struct ResponseBytesBuffer {
        response_bytes: Buffer
    }

    impl Default for ResponseBytesBuffer {
        fn default() -> Self {
            Self {
                response_bytes: Buffer::with_capacity(0)
            }
        }
    }

    impl HttpResponse {
        pub fn write_to_cxn(& mut self, cxn: & dyn Connection<&[u8]>) {
            while !self.response_bytes.response_bytes.empty() {
                let mut more_bytes: [u8; 4096] = [0; 4096];
                self.response_bytes.response_bytes.read(more_bytes.as_mut());
                cxn.write_bytes(more_bytes);
            }
        }
    }

    pub trait ResponseWriter<T> {
        fn write(&mut self, response: T);
    }

    impl <'a> ResponseWriter<&[u8]> for HttpResponse {
        fn write(&mut self, response: &[u8]) {
            self.response = String::from_utf8(Vec::from(response))
                .map(|response_str| self.response.clone() + response_str.as_str())
                .unwrap_or(self.response.clone());
            self.response_bytes = ResponseBytesBuffer {response_bytes: Buffer::from_slice(response)}
        }
    }

    impl <'a> ResponseWriter<&[u8]> for HttpRequest {
        fn write(&mut self, response: &[u8]) {
            todo!()
        }
    }

    impl Default for HttpRequest {
        fn default() -> Self {
            Self {
                headers: HashMap::new(),
                body: String::default(),
                metadata: EndpointMetadata::default(),
                method: HttpMethod::Get,
                // connection: None,
                // bytes: &[0]
            }
        }
    }

    impl <'a> Default for HttpResponse {
        fn default() -> Self {
            Self {
                session: HttpSession::default(),
                response: String::default(),
                response_bytes: ResponseBytesBuffer::default()
            }
        }
    }
}
