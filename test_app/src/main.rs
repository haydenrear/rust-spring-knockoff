use std::marker::PhantomData;
use lazy_static::lazy_static;
use hyper::{HyperRequestConverter, HyperRequestStream};
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use data_framework::Entity;
use knockoff_security::knockoff_security::user_request_account::{AccountData, UserAccount, UserSession};
use web_framework::{create_message_converter, default_message_converters};
use web_framework::web_framework::convert::{EndpointRequestExtractor, MessageConverter, Registration};
use web_framework::web_framework::dispatch::FilterExecutor;
use web_framework::web_framework::filter::filter::{Filter, FilterChain};
use web_framework_shared::request::WebResponse;
use web_framework::web_framework::security::user_details::PersistenceUserDetailsService;
use web_framework::web_framework::http::RequestExecutorImpl;
use web_framework::web_framework::context::{Context, RequestHelpers};
use web_framework::web_framework::context_builder::{ApplicationContextBuilder, AuthenticationConverterRegistryBuilder, ConverterRegistryBuilder, DelegatingAuthenticationManagerBuilder, FilterRegistrarBuilder, RequestContextBuilder};
use web_framework::web_framework::message::MessageType;
use web_framework_shared::request::{EndpointMetadata, WebRequest};
use module_macro_lib::AuthenticationTypeConverterImpl;
use mongo_repo::{Db, MongoRepo};
use web_framework::web_framework::request_context::RequestContext;
use web_framework::web_framework::security::authentication::{AuthenticationProvider, AuthenticationToken, DaoAuthenticationProvider};
use web_framework::web_framework::security::password::NoOpPasswordEncoder;
use web_framework::web_framework::session::session::HttpSession;
use web_framework_shared::authority::GrantedAuthority;
use web_framework_shared::dispatch_server::Handler;

#[derive(Serialize, Deserialize, Clone)]
pub struct TestUserAccount;

impl Entity<String> for TestUserAccount {
    fn get_id(&self) -> Option<String> {
        Some("test".to_string())
    }

    fn set_id(&mut self, id: String) {
    }
}

impl UserAccount for TestUserAccount {
    fn get_account_data(&self) -> AccountData {
        todo!()
    }

    fn login(&self) {
    }

    fn get_password(&self) -> String {
        "test".to_string()
    }
}

#[test]
fn test(){}

#[tokio::main]
async fn main() {
    let filter: Filter<Example, Example> = Filter::new(
        Arc::new(TestAction::default()), None, Arc::new(FilterExecutor::default())
    );
    let mut filter_registrar = FilterRegistrarBuilder {
        filters: Arc::new(Mutex::new(vec![])),
        already_built: false,
        fiter_chain: Arc::new(FilterChain::default())
    };

    default_message_converters!();

    create_message_converter!(
        (crate::NewConverter1 => NewConverter1{} =>> "custom/convert1" => NewConverter1 => new_converter_1)
        ===> Example => DelegatingMessageConverter
    );

    create_message_converter!(
        (crate::NewConverter3 => NewConverter3{} =>> "custom/convert1" => NewConverter3 => new_converter)
        ===> Example1 => ExampleDelegatingMessageConverter
    );


    filter_registrar.register(filter);
    let ctx_builder = ApplicationContextBuilder::<Example, Example> {
        filter_registry: Some(Arc::new(Mutex::new(filter_registrar))),
        request_context_builder: Some(Arc::new(Mutex::new(RequestContextBuilder {
            message_converter_builder: ConverterRegistryBuilder {
                converters: Arc::new(Mutex::new(Some(Box::new(DelegatingMessageConverter::new())))),
                request_convert: Arc::new(Mutex::new(Some(Box::new(EndpointRequestExtractor{}))))
            },
            authentication_manager_builder: DelegatingAuthenticationManagerBuilder {
                providers: Arc::new(Mutex::new(vec![].into())),
            },
        }))),
        authentication_converters: Some(Arc::new(AuthenticationConverterRegistryBuilder {
            converters: Arc::new(Mutex::new(vec![])),
            authentication_type_converter: Arc::new(Mutex::new(Some(Box::new(AuthenticationTypeConverterImpl {}))))
        })),
    };
    let mut r: HyperRequestStream<Example, Example> = HyperRequestStream::new(
        RequestExecutorImpl {
            ctx: ctx_builder.build()
        }
    );

    r.do_run();

    let filter1: Filter<Example1, Example1> = Filter::new(
        Arc::new(TestAction::default()), None, Arc::new(FilterExecutor::default())
    );
    let mut filter_registrar1 = FilterRegistrarBuilder {
        filters: Arc::new(Mutex::new(vec![])),
        already_built: false,
        fiter_chain: Arc::new(FilterChain::default())
    };
    filter_registrar1.register(filter1);
    let ctx_builder1 = ApplicationContextBuilder::<Example1, Example1> {
        filter_registry: Some(Arc::new(Mutex::new(filter_registrar1))),
        request_context_builder: Some(Arc::new(Mutex::new(RequestContextBuilder {
            message_converter_builder: ConverterRegistryBuilder {
                converters: Arc::new(Mutex::new(Some(Box::new(ExampleDelegatingMessageConverter::new())))),
                request_convert: Arc::new(Mutex::new(Some(Box::new(EndpointRequestExtractor{}))))
            },
            authentication_manager_builder: DelegatingAuthenticationManagerBuilder {
                providers: Arc::new(Mutex::new(vec![Box::new(DaoAuthenticationProvider {
                    user_details_service: PersistenceUserDetailsService::<MongoRepo, TestUserAccount> {
                        p: Default::default(),
                        u: Default::default(),
                        repo: MongoRepo::new("user_details", "user_details"),
                    },
                    password_encoder: Box::new(NoOpPasswordEncoder{}),
                    phantom_user: Default::default(),
                })])),
            },
        }))),
        authentication_converters: Some(Arc::new(AuthenticationConverterRegistryBuilder {
            converters: Arc::new(Mutex::new(vec![])),
            authentication_type_converter: Arc::new(Mutex::new(Some(Box::new(AuthenticationTypeConverterImpl {}))))
        })),
    };

    let mut r: HyperRequestStream<Example1, Example1> = HyperRequestStream::new(
        RequestExecutorImpl {
            ctx: ctx_builder1.build()
        }
    );

    r.do_run().await;

}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Example {
    value: String
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Example1 {
    value: String
}

impl Default for Example {
    fn default() -> Self {
        Example {
            value: String::from("hello!"),
        }
    }
}

struct TestMessageConverter;

#[derive(Serialize, Deserialize)]
struct TestJson {
    value: String,
}

struct TestAction1;
impl Handler<Example1, Example1, RequestContext, Context<Example1, Example1>> for TestAction {
    fn do_action(
        &self,
        request: &Option<Example1>,
        web_request: &WebRequest,
        response: &mut WebResponse,
        ctx: &Context<Example1, Example1>,
        request_context: &mut RequestContext
    ) -> Option<Example1> {
        Some(Example1::default())
    }

    fn authentication_granted(&self, token: &Vec<GrantedAuthority>) -> bool {
        true
    }

    fn matches(&self, endpoint_metadata: &EndpointMetadata) -> bool {
        true
    }

}


struct TestAction;
impl Handler<Example, Example, RequestContext, Context<Example, Example>> for TestAction {
    fn do_action(
        &self,
        request: &Option<Example>,
        web_request: &WebRequest,
        response: &mut WebResponse,
        ctx: &Context<Example, Example>,
        request_context: &mut RequestContext
    ) -> Option<Example> {
        Some(Example::default())
    }

    fn authentication_granted(&self, token: &Vec<GrantedAuthority>) -> bool {
        true
    }

    fn matches(&self, endpoint_metadata: &EndpointMetadata) -> bool {
        true
    }

}

impl Clone for TestAction {
    fn clone(&self) -> Self {
        Self
    }
}

impl Default for TestAction {
    fn default() -> Self {
        Self
    }
}

#[derive(Clone)]
pub struct NewConverter {

}

impl <Request, Response> MessageConverter<Request, Response> for NewConverter
    where
        Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
        Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static
{
    fn new() -> Self where Self: Sized {
        todo!()
    }

    fn convert_to(&self, request: &WebRequest) -> Option<MessageType<Request>> {
        todo!()
    }

    fn convert_from(&self, request_body: &Response, request: &WebRequest) -> Option<String> {
        todo!()
    }

    fn do_convert(&self, request: &WebRequest) -> bool {
        todo!()
    }

    fn message_type(&self) -> Vec<String> {
        vec!["".to_string()]
    }
}

#[derive(Clone)]
pub struct NewConverter1 {
}

impl MessageConverter<Example, Example> for NewConverter1
{
    fn new() -> Self where Self: Sized {
        todo!()
    }

    fn convert_to(&self, request: &WebRequest) -> Option<MessageType<Example>> {
        todo!()
    }

    fn convert_from(&self, request_body: &Example, request: &WebRequest) -> Option<String> {
        todo!()
    }

    fn do_convert(&self, request: &WebRequest) -> bool {
        todo!()
    }

    fn message_type(&self) -> Vec<String> {
        vec!["".to_string()]
    }
}

#[derive(Clone)]
pub struct NewConverter3;
impl MessageConverter<Example1, Example1> for NewConverter3 {
    fn new() -> Self where Self: Sized {
        Self {}
    }

    fn convert_to(&self, request: &WebRequest) -> Option<MessageType<Example1>> {
        todo!()
    }

    fn convert_from(&self, request_body: &Example1, request: &WebRequest) -> Option<String> {
        todo!()
    }

    fn do_convert(&self, request: &WebRequest) -> bool {
        todo!()
    }

    fn message_type(&self) -> Vec<String> {
        todo!()
    }
}
