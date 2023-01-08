pub mod request {

    use crate::web_framework::context::RequestContext;
    use crate::web_framework::http::{Connection, HttpMethod};
    use crate::web_framework::session::session::HttpSession;
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
        pub http_method: HttpMethod
    }

    impl Default for EndpointMetadata {
        fn default() -> Self {
            Self {
                path_variables: String::default(),
                query_params: String::default(),
                http_method: HttpMethod::Get
            }
        }
    }

    trait HttpEntity {}


    #[derive(Serialize, Deserialize)]
    pub struct WebRequest {
        pub headers: HashMap<String, String>,
        pub body: String,
        pub metadata: EndpointMetadata,
        pub method: HttpMethod,
    }

    impl Clone for WebRequest {
        fn clone(&self) -> Self {
            Self {
                headers: self.headers.clone(),
                body: self.body.clone(),
                metadata: self.metadata.clone(),
                method: HttpMethod::Get,
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

    #[derive(Clone, Serialize, Deserialize)]
    pub struct WebResponse {
        pub session: HttpSession,
        pub response: String,
        #[serde(skip_serializing, skip_deserializing)]
        pub response_bytes: ResponseBytesBuffer
    }

    impl WebResponse {
        pub fn response_bytes(&mut self) -> Result<Vec<u8>, String> {
            self.response_bytes.read_and_empty_buffer()
        }
    }

    #[derive(Clone)]
    pub struct ResponseBytesBuffer {
        response_bytes: Buffer
    }

    impl ResponseBytesBuffer {
        const SIZE: usize = 4096;
        fn add_bytes(&mut self, mut bytes: &[u8]) {
            self.response_bytes.write(bytes);
        }

        fn next(&mut self) -> [u8; Self::SIZE] {
            let mut more_bytes: [u8; Self::SIZE] = [0; Self::SIZE];
            self.response_bytes.read(more_bytes.as_mut());
            more_bytes
        }

        fn empty(&self) -> bool {
            self.response_bytes.empty()
        }

        pub fn write(&mut self, to_write: &[u8]) -> std::io::Result<usize> {
            if self.response_bytes.available_space() < to_write.len()  {
                self.response_bytes.grow(self.response_bytes.capacity() + to_write.len() * 2);
            }
            self.response_bytes.write(to_write)
        }

        pub fn read_and_empty_buffer(&mut self) -> Result<Vec<u8>, String> {
            let mut created_vec = vec![0; self.response_bytes.available_data()];
            let mut response: &mut [u8] = created_vec.as_mut_slice();
            self.response_bytes.read(response)
                .map(|r| created_vec)
                .or(Err("Error reading from buffer".to_string()))
        }

    }

    impl Default for ResponseBytesBuffer {
        fn default() -> Self {
            Self {
                response_bytes: Buffer::with_capacity(12000)
            }
        }
    }

    pub trait ResponseWriter<T> {
        fn write(&mut self, response: T);
    }

    impl <'a> ResponseWriter<&[u8]> for WebResponse {
        fn write(&mut self, response: &[u8]) {
            self.response = String::from_utf8(Vec::from(response))
                .map(|response_str| self.response.clone() + response_str.as_str())
                .unwrap_or(self.response.clone());
            self.response_bytes.write(response);
        }
    }

    impl <'a> ResponseWriter<&[u8]> for WebRequest {
        fn write(&mut self, response: &[u8]) {
            todo!()
        }
    }

    impl Default for WebRequest {
        fn default() -> Self {
            Self {
                headers: HashMap::new(),
                body: String::default(),
                metadata: EndpointMetadata::default(),
                method: HttpMethod::Get,
            }
        }
    }

    impl <'a> Default for WebResponse {
        fn default() -> Self {
            Self {
                session: HttpSession::default(),
                response: String::default(),
                response_bytes: ResponseBytesBuffer::default()
            }
        }
    }
}
