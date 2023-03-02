use std::ops::Deref;
use quote::ToTokens;
use syn::Item;
use syn::parse::Parse;
use codegen_utils::parse;
use crate::aspect::AspectAwarePackage;

#[test]
fn test_create_aspect_aware_package() {
    let syn_file = parse::open_syn_file(
        "/Users/hayde/IdeaProjects/rust-spring-knockoff/codegen_resources",
        "test_library_three.rs"
    )
        .unwrap();

    if &syn_file.items.len() < &1 {
        assert!(false);
    }

    let mut did_run = false;
    let _ = syn_file.items.iter().for_each(|i| {
        match i {
            Item::Mod(item_mod) => {
                let mut count = 0;
                let vec = item_mod.content.clone().unwrap().1;
                while !did_run && count < vec.len() {
                    let item = &vec[count];
                    match item {
                        Item::Impl(impl_item) => {
                            let string = impl_item.self_ty.to_token_stream().clone().to_string();
                            let x = string.as_str();
                            println!("{}", x);
                            if x.contains("One") {
                                let path = impl_item.trait_.clone().unwrap().1;
                                let aap = AspectAwarePackage::new(&path.clone());
                                let module_path = aap.module_path;
                                println!("{} is the module path", module_path);
                                did_run = true;
                            }
                        }
                        _ => {
                        }
                    }
                    count += 1;
                }

            }
            _ => {}
        };
    });
    assert!(did_run);
}