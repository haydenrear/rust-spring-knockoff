pub(crate) mod type_mod {

    pub trait GetType {
        fn get_type(name: String) -> HTypeId;
        fn get_type_self(&self) -> HTypeId;
    }

    pub struct HTypeId {
        name: String,
    }

    impl PartialEq for HTypeId {
        fn eq(&self, other: &Self) -> bool {
            self.name == other.name
        }
    }

    impl HTypeId {
        pub(crate) fn new(name: String) -> Self {
            Self { name: name }
        }
    }
}
