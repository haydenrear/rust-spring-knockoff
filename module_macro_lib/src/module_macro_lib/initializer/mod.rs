use crate::{FieldAugmenterImpl, ContextInitializerImpl};

#[derive(Default, Clone, Debug)]
pub struct ModuleMacroInitializer {
    pub field_augmenter: FieldAugmenterImpl,
    pub initialize: ContextInitializerImpl
}