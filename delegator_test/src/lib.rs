#![feature(proc_macro_hygiene)]
use delegator_macro_rules::{last_thing, types};
use module_macro::module_attr;
use std::fmt::Display;
use crate::test_library::*;

use syn::{
    parse_macro_input,
    LitStr,
    Token,
    Ident,
    token::Paren,
    DeriveInput
};

use quote::{quote, format_ident, IdentFragment, ToTokens};
use crate::test_library::test_library_two::test_library_two::Three;

#[module_attr]
pub mod test_library {


    use module_macro::module_attr;
    use syn::{parse_macro_input, LitStr, Token, Ident, token::Paren, DeriveInput, ItemStruct, Fields, Field};

    use quote::{quote, format_ident, IdentFragment, ToTokens};
    use derive_syn_parse::Parse;
    use syn::parse::{Parse, Parser, ParseStream};
    use rust_spring_macro::module_post_processor::ModuleStructPostProcessor;

    #[path = "test_library_two.rs"]
    pub mod test_library_two;


    pub trait Found {
    }

    impl Found for One {
    }

    impl One {
        fn new() -> Self {
            Self {
                a: String::from(""),
                two: String::default()
            }
        }
    }

    pub struct Four<'a> {
        one: &'a [String]
    }

    pub struct One {
        pub(crate) two: String
    }


}

#[test]
fn test() {
    use test_library::One;
    let one: One = One{
        a: String::from("hello"),
        two: String::default()
    };
    // let three: Three = Three{a: String::from("hello")};
}

