use crate::web_framework::session::session::HttpSession;

#[derive(Default, Clone)]
pub struct RequestContext {
    pub http_session: HttpSession
}