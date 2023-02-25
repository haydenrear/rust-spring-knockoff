use module_macro::{field_aug, initializer, module_attr};

#[initializer]
pub fn example_initializer() {
    let listable: ListableBeanFactory = AbstractListableFactory::<DefaultProfile>::new();
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

#[authentication_type]
#[cfg(authentication_type)]
pub mod authentication_type {

    #[derive(Default, Clone, Debug, Serialize, Deserialize)]
    pub struct TestAuthType {
        some_token: String
    }

    impl AuthType for TestAuthType {
        const AUTH_TYPE: &'static str = "test_auth_type";

        fn parse_credentials(request: &WebRequest) -> Result<Self, AuthenticationConversionError> {
            Ok(TestAuthType::default())
        }
    }

    impl AuthenticationAware for TestAuthType {
        fn get_authorities(&self) -> LinkedList<Authority> {
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

pub fn get_authentication_type() {
}
