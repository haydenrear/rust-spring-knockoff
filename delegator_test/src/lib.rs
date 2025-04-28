#![feature(proc_macro_hygiene)]

use std::any::TypeId;
use std::fmt::Display;

use quote::{format_ident, IdentFragment, quote, ToTokens};
use syn::{DeriveInput, Field, Fields, Ident, ItemStruct, LitStr, parse_macro_input, Token, token::Paren};
use syn::token::Type;

use module_macro::{module_attr};

use std::any::Any;
use std::sync::{Arc};
use std::collections::HashMap;
use std::ops::Deref;
use std::marker::PhantomData;
use syn::parse::Parser;
use module_macro_lib::module_macro_lib::knockoff_context_builder::bean_constructor_generator::BeanConstructorGenerator;
use module_macro_shared::bean::BeanPathParts::PhantomType;
use module_macro_shared::profile_tree::ProfileBuilder as ModuleProfile;
use module_precompile_macro::boot_knockoff;

#[module_attr]
pub mod test_library {
    use spring_knockoff_boot_macro::*;

    pub mod test_library_two;
    pub use test_library_two::*;
    pub mod test_library_three;
    pub use test_library_three::*;
    pub mod test_library_seven;
    pub use test_library_seven::*;

    #[aspect(test_library.test_library_three.test_library_four.One.*)]
    #[ordered(0)]
    #[cfg(springknockoff)]
    pub fn do_aspect(&self, one: One) -> String {
        println!("hello");
        println!("{}", self.two.clone());
        let found = self.proceed(one);
        let mut three_four = "four three ".to_string() + found.as_str();
        three_four
    }

    #[aspect(test_library.test_library_three_test_library_four.One.*)]
    #[ordered(1)]
    #[cfg(springknockoff)]
    pub fn do_aspect_again(&self, one: One) -> String {
        println!("hello");
        println!("{}", self.two.clone());
        let found = self.proceed(one);
        let mut zero = " zero".to_string();
        zero = found + zero.as_str();
        zero
    }


    #[processor]
    #[authentication_type]
    #[cfg(authentication_type)]
    pub mod authentication_type {
        use spring_knockoff_boot_macro::*;
        use serde::{Serialize, Deserialize};
        use web_framework_shared::*;
        use knockoff_security::{AuthType, AuthenticationConversionError, AuthenticationAware};

        #[auth_type_struct(TestAuthType)]
        #[derive(Default, Clone, Debug, Serialize, Deserialize)]
        #[knockoff_ignore]
        pub struct TestAuthType {
            // some_token: String
        }

        #[auth_type_impl(TestAuthType)]
        #[knockoff_ignore]
        impl TestAuthType {
            const AUTH_TYPE: &'static str = "test_auth_type";

            pub fn parse_credentials(request: &WebRequest) -> Result<Self, AuthenticationConversionError> {
                Ok(TestAuthType::default())
            }
        }

        #[auth_type_aware(TestAuthType)]
        #[knockoff_ignore]
        impl TestAuthType {
            pub fn get_authorities(&self) -> Vec<GrantedAuthority> {
                todo!()
            }

            pub fn get_credentials(&self) -> Option<String> {
                todo!()
            }

            pub fn get_principal(&self) -> Option<String> {
                todo!()
            }

            pub fn set_credentials(&mut self, credential: String) {
                todo!()
            }

            pub fn set_principal(&mut self, principal: String) {
                todo!()
            }
        }

    }

}

pub use test_library::*;
use knockoff_security::knockoff_security::*;
use web_framework::{AuthenticationType, TestAuthType as FrameworkTestAuthType};


#[test]
fn test_module_macro() {
    create_with_extra_field();
    let j = AuthenticationType::TestAuthType( FrameworkTestAuthType {} );

    let listable: ListableBeanFactory = AbstractListableFactory::<DefaultProfile>::new();
    assert_ne!(listable.singleton_bean_definitions.len(), 0);

    let four_found_again: Option<Arc<TestLibraryFourAgain>> = BeanContainer::<TestLibraryFourAgain>::fetch_bean(&listable);
    assert!(four_found_again.is_some());

    let four_found_again: Option<Arc<Four>> = BeanContainer::<Four>::fetch_bean(&listable);
    four_found_again.unwrap().one.lock().unwrap().two = "another".to_string();

    let one_found: Option<Arc<Ten>> = BeanContainer::<Ten>::fetch_bean(&listable);
    let two_found: Option<Arc<Ten>> = BeanContainer::<Ten>::fetch_bean(&listable);

    assert!(two_found.is_some());
    assert!(one_found.is_some());

    let four_found: Option<Arc<Four>> = BeanContainer::<Four>::fetch_bean(&listable);
    let one_found_again: Option<Arc<One>> = BeanContainer::<One>::fetch_bean(&listable);
    assert!(four_found.as_ref().is_some());
    assert_eq!(four_found.unwrap().one.lock().unwrap().deref().type_id().clone(), one_found_again.as_ref().unwrap().deref().type_id());

    let four_found_third: Option<Arc<Four>> = BeanContainer::<Four>::fetch_bean(&listable);
    assert_eq!(four_found_third.unwrap().one.lock().unwrap().two, "another".to_string());

    let found = BeanContainer::<dyn Found>::fetch_bean(&listable);
    assert!(found.is_some());
    let found = BeanContainerProfile::<dyn Found, DefaultProfile>::fetch_bean_profile(&listable);
    assert!(found.is_some());

    let once_found: Option<Arc<Once>> = BeanContainer::<Once>::fetch_bean(&listable);
    assert!(once_found.is_some());

    let mutable_bean_one = BeanContainer::<Mutex<Box<dyn Found>>>::fetch_bean(&listable);
    assert!(mutable_bean_one.is_some());

    let one = PrototypeBeanContainer::<One>::fetch_bean(&listable);

    assert!(one_found_again.as_ref().is_some());
    let wrapped = one_found_again.unwrap().one_two_three(One { two: "".to_string() });

    // assert_eq!(wrapped, "four three two one zero".to_string());

    let with_generics = BeanContainer::<TestWithGenerics>::fetch_bean(&listable);
    assert!(with_generics.is_some(), "Failed to get non-dyn");

    let with_generics = BeanContainer::<dyn HasEnum<TestConstructEnumWithFields>>::fetch_bean(&listable);
    assert!(with_generics.is_some(), "Failed to get dyn");

    let created_enum = BeanContainer::<TestConstructEnumWithFields>::fetch_bean(&listable);
    assert!(created_enum.is_some(), "Failed to create enum.");

    let created_enum = BeanContainer::<TestWithGenericsInStruct>::fetch_bean(&listable);
    assert!(created_enum.is_some(), "Failed to create enum.");

    let created_enum_one: Option<Arc<TestInjectContainsPhantom>> = BeanContainer::<TestInjectContainsPhantom>::fetch_bean(&listable);
    assert!(created_enum_one.as_ref().is_some(), "Failed to create enum.");

    let created_enum_two: Option<Arc<TestInjectContainsPhantom>> = BeanContainer::<TestInjectContainsPhantom>::fetch_bean(&listable);
    assert!(created_enum_two.as_ref().is_some(), "Failed to create enum.");

    let phantom_created = created_enum_one.as_ref().unwrap().contains_phantom.type_id();
    assert_eq!(phantom_created, created_enum_two.unwrap().contains_phantom.type_id());

    let phantom_found = BeanContainer::<ContainsPhantom<TestT, TestU, TestV, TestV>>::fetch_bean(&listable).unwrap();
    assert_eq!(phantom_found.type_id(), phantom_created);

    let concrete_prototype = BeanContainer::<TestInjectPrototypeBean>::fetch_bean(&listable);
    assert!(concrete_prototype.is_some());

    let prototype_bean = PrototypeBeanContainer::<TestPrototypeBean>::fetch_bean(&listable);

    let concrete_prototype = BeanContainer::<TestInjectPrototypeBeanFromFactoryFn>::fetch_bean(&listable);
    assert!(concrete_prototype.is_some());

    let prototype_bean = PrototypeBeanContainer::<TestPrototypeBeanFromFactoryFn>::fetch_bean(&listable);

    let concrete_prototype = BeanContainer::<Mutex<TestInjectPrototypeBeanFromFactoryFn>>::fetch_bean(&listable);
    assert!(concrete_prototype.is_some());

}


#[test]
fn test_attribute_handler_mapping() {
    let attr = AttributeHandlerMapping::new();
}

#[test]
fn test_app_ctx() {
    let app_ctx = AppCtx::new();
    assert_eq!(app_ctx.profiles.len(), 1);
    assert!(app_ctx.profiles.iter().any(|p| p == &ModuleProfile::default().profile));
    let found = app_ctx.get_bean::<One>();
    assert!(found.is_some());
    let found = app_ctx.get_bean_for_profile::<One, DefaultProfile>();
    assert!(found.is_some());
    let found = app_ctx.get_bean_for_profile::<Mutex<Box<dyn Found>>, DefaultProfile>();
    assert!(found.is_some());
}

fn create_with_extra_field() {
    let ten = Ten {
    };

    let one: One = One {
        two: String::default()
    };

}