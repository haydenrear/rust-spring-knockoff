use futures::TryStreamExt;
use serde::{Deserialize, Serialize};
use crate::context::Context;
use crate::convert::ConverterContext;
use crate::filter::filter::{Action, MediaType};
use crate::message::MessageType;
use crate::request::request::{EndpointMetadata, HttpRequest, HttpResponse, RequestExtractor, ResponseWriter};

pub struct Dispatcher {
    pub context: Context
}

impl Dispatcher {
    pub(crate) fn do_request<'a, Response, Request>(&self, request: HttpRequest,
                                                    response: &mut HttpResponse,
                                                    action: &Box<dyn Action<Request, Response>>)
    where
        Response: Serialize + for<'b> Deserialize<'b> + Clone + Default,
        Request: Serialize + for<'b> Deserialize<'b> + Clone + Default
    {
        self.context.convert_to(&request)
            .and_then(|found| {
                self.context.convert_extract(&request)
                    .and_then(|metadata|
                        action.do_action(
                            metadata,
                            &found.message,
                            &self.context,
                        )
                    )
                    .and_then(|action_response|
                        self.context.convert_from(
                            &found.message,
                            MediaType::Json
                        ))
            })
            .map(|response_to_write| {
                response.write(response_to_write.clone().as_bytes());
                response_to_write
            });
    }
}

impl Default for Dispatcher {
    fn default() -> Self {
        Self {
            context: Context::default()
        }
    }
}

pub trait RequestMethodDispatcher<Response, Request> {
    fn do_method(&self) -> dyn Fn(EndpointMetadata, Request, &Context) -> Response;
}

pub struct PostMethodRequestDispatcher {
    pub context: Context
}

impl Default for PostMethodRequestDispatcher {
    fn default() -> Self {
        Self {
            context: Context::default()
        }
    }
}

