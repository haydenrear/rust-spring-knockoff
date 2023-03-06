#[macro_export]
macro_rules! codegen_items {
    ($(($codegen_item:ty, $codegen_ident:ident)),*) => {

        #[derive(Clone)]
        pub enum CodegenItemType {
            $(
                $codegen_ident($codegen_item)
            ),*
        }

        impl CodegenItem for CodegenItemType {

            fn supports_item(item: &Vec<Item>) -> bool where Self: Sized {
                let mut supports = false;
                $(
                    supports = <$codegen_item>::supports_item(item);
                )*
                supports
            }

            fn supports(&self, item: &Vec<Item>) -> bool {
                match self {
                    $(
                        CodegenItemType::$codegen_ident(codegen_item) => {
                            codegen_item.supports(item)
                        }
                    )*
                }
            }

            fn get_codegen(&self) -> Option<String> {
                match self {
                    $(
                        CodegenItemType::$codegen_ident(codegen_item) => {
                            codegen_item.get_codegen()
                        }
                    )*
                }
            }

            fn default_codegen(&self) -> String {
                match self {
                    $(
                        CodegenItemType::$codegen_ident(codegen_item) => {
                            codegen_item.default_codegen()
                        }
                    )*
                }
            }

            fn get_unique_id(&self) -> String {
                match self {
                    $(
                        CodegenItemType::$codegen_ident(codegen_item) => {
                            codegen_item.get_unique_id()
                        }
                    )*
                }
            }

            fn get_unique_ids(&self) -> Vec<String> {
                match self {
                    $(
                        CodegenItemType::$codegen_ident(codegen_item) => {
                            codegen_item.get_unique_ids()
                        }
                    )*
                }
            }
        }

        pub fn get_codegen_item(item: &Vec<Item>) -> Vec<CodegenItemType> {
            let mut codegen_items = vec![];
            $(
                if <$codegen_item>::supports_item(&item) {
                    let codegen_item: Option<CodegenItemType> = <$codegen_item>::new_dyn_codegen(item);
                    codegen_item.map(|codegen_item| codegen_items.push(codegen_item));
                }
            )*
            codegen_items
        }

        impl LibParser {
            pub fn gen_codegen_items() -> CodegenItems {
                let mut codegen: Vec<CodegenItemType>  = vec![];
                $(
                    let codegen_item = <$codegen_item>::default();
                    if codegen_item.get_unique_id().len() != 0 {
                        codegen.push(CodegenItemType::$codegen_ident(codegen_item));
                    }
                )*
                CodegenItems {
                    codegen
                }
            }
        }

    }
}
