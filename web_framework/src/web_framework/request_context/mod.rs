use crate::web_framework::session::session::HttpSession;

#[derive(Default, Clone)]
pub struct SessionContext {
    pub http_session: HttpSession
}