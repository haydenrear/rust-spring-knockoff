#[macro_export]
macro_rules! codegen_items {
    ($($codegen_item:ty),*) => {
        pub fn get_codegen_item(item: &Vec<Item>) -> Vec<Box<dyn CodegenItem>> {
            let mut codegen_items = vec![];
            $(
                if <$codegen_item>::supports_item(&item) {
                    let codegen_item: Option<Box<dyn CodegenItem>> = <$codegen_item>::new_dyn_codegen(item);
                    codegen_item.map(|codegen_item| {
                        codegen_items.push(codegen_item);
                    });
                }
            )*
            codegen_items
        }

        pub fn get_codegen_item_any(item: &Vec<Item>) -> Vec<Box<dyn Any>> {
            let mut codegen_items = vec![];
            $(
                if <$codegen_item>::supports_item(&item) {
                    let codegen_item: Option<Box<dyn Any>> = <$codegen_item>::new_any(item);
                    codegen_item.map(|codegen_item| {
                        codegen_items.push(codegen_item);
                    });
                }
            )*
            codegen_items
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
