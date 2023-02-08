use module_macro::{bean, singleton, autowired};

#[derive(Default, Debug)]
#[singleton(Once)]
pub struct Ten {}