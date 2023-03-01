pub mod module_macro_shared_codegen;
pub mod aspect;

#[cfg(test)]
mod tests {
    use super::*;
    use module_macro_shared_codegen::ExampleContextInitializer;

    #[test]
    pub fn it_works() {
        let ex = ExampleContextInitializer{};
    }

}
