mod test;

mod static_registrar {
    use std::any::Any;
    use std::collections::HashMap;
    use lazy_static::lazy_static;
    use std::sync::{Arc, Mutex, MutexGuard};
    use async_std::sync::Arc as Rc;
    use crate::convert::Registration;

}