use module_macro::{field_aug, initializer, module_attr};

#[initializer]
pub fn example_initializer() {
    println!("Hello...");
}

#[field_aug]
pub fn field_aug(struct_item: &mut ItemStruct) {
    match &mut struct_item.fields {
        Fields::Named(ref mut fields_named) => {
            fields_named.named.push(
                Field::parse_named.parse2(quote!(
                                    pub a: String
                                ).into()).unwrap()
            )
        }
        Fields::Unnamed(ref mut fields_unnamed) => {}
        _ => {}
    }
}

#[aspect(**)]
#[ordered(0)]
pub fn do_aspect(&self, one: One) -> String {
    println!("hello");
    println!("{}", self.two.clone());
    let found = self.proceed_one_test(one);
    found
}

#[aspect(**)]
#[ordered(1)]
pub fn second_aspect(&self, one: One) -> String {
    println!("another_hello");
    println!("{}", self.two.clone());
    let found = self.proceed_one_test_again(one);
    "another".to_string()
}


#[configuration(field_aug)]
pub mod field_aug_configuration {

}

#[authentication_type]
#[cfg(authentication_type)]
pub mod authentication_type {

    #[derive(Default, Clone, Debug, Serialize, Deserialize)]
    #[auth_type_struct(TestAuthType)]
    pub struct TestAuthType {
        some_token: String
    }

    #[auth_type_impl(TestAuthType)]
    impl AuthType for TestAuthType {
        const AUTH_TYPE: &'static str = "test_auth_type";

        fn parse_credentials(request: &WebRequest) -> Result<Self, AuthenticationConversionError> {
            Ok(TestAuthType::default())
        }
    }

    #[auth_type_aware(TestAuthType)]
    impl AuthenticationAware for TestAuthType {
        fn get_authorities(&self) -> Vec<GrantedAuthority> {
            todo!()
        }

        fn get_credentials(&self) -> Option<String> {
            todo!()
        }

        fn get_principal(&self) -> Option<String> {
            todo!()
        }

        fn set_credentials(&mut self, credential: String) {
            todo!()
        }

        fn set_principal(&mut self, principal: String) {
            todo!()
        }
    }

}