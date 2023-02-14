use crate::{ContextInitializerImpl, FieldAugmenterImpl};

#[derive(Default, Clone, Debug)]
pub struct Initializer {
    pub field_augmenter: FieldAugmenterImpl,
    pub initializer: ContextInitializerImpl
}