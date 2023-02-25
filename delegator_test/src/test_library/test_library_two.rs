use spring_knockoff_boot_macro::{autowired, bean, singleton};

#[derive(Default, Debug)]
#[singleton(Once)]
pub struct Ten {}