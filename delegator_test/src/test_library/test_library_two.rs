use module_macro::{bean, singleton, Component, autowired};

#[derive(Default, Debug)]
#[singleton(Once)]
pub struct Ten {}