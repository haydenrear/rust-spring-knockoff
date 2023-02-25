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

    #[derive(Default, Clone, Serialize, Deserialize)]
    pub struct TestAuthType {
        some_token: String
    }

    impl AuthType for TestAuthType {
        fn parse_credentials(&self, request: &WebRequest) -> Result<TestAuthType, AuthenticationConversionError> {
            Ok(TestAuthType::default())
        }
    }

}

pub fn get_authentication_type() {
}
