pub mod test_library_six;

use std::fmt::{Debug, Formatter};
use spring_knockoff_boot_macro::{autowired, bean, singleton};
use crate::test_library::test_library_three::One;

#[derive(Default, Debug)]
#[singleton(Once)]
pub struct Ten {}

