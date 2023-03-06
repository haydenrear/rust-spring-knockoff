#[macro_export]
macro_rules! codegen_items {
    ($($codegen_item:ty),*) => {
        pub fn get_codegen_item(item: &Item) -> Option<Box<dyn CodegenItem>> {
            $(
                if <$codegen_item>::supports_item(item) {
                    let codegen_item: Option<Box<dyn CodegenItem>> = <$codegen_item>::new_dyn_codegen(item);
                    return codegen_item;
                }
            )*
            None
        }

        pub fn get_codegen_item_any(item: &Item) -> Option<Box<dyn Any>> {
            $(
                if <$codegen_item>::supports_item(item) {
                    let codegen_item: Option<Box<dyn Any>> = <$codegen_item>::new_any(item);
                    return codegen_item;
                }
            )*
            None
        }

        impl LibParser {
            pub fn gen_codegen_items() -> CodegenItems {
                let mut codegen: Vec<Box<dyn CodegenItem>>  = vec![];
                $(
                    let codegen_item = <$codegen_item>::default();
                    if codegen_item.get_unique_id().len() != 0 {
                        codegen.push(Box::new(codegen_item));
                    }
                )*
                CodegenItems {
                    codegen
                }
            }
        }

    }
}
