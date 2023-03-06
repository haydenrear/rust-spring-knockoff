use std::env;
use std::env::VarError;
use std::fs::File;
use std::path::Path;
use syn::{Attribute, Item, ItemFn};
use crate::syn_helper::SynHelper;

#[test]
fn test_syn_helper() {
    let out = do_test(&|attribute| {
        let attr = SynHelper::parse_attr_path_single(&attribute);
        attr.is_some() && attr.unwrap() == "*"
    });
    assert!(out.is_some());
}

#[test]
fn test_get_attr_from_vec() {
    let out = do_test(&|attribute| {
        let attr = SynHelper::get_attr_from_vec(&vec![attribute.clone()], vec!["aspect"]);
        println!("{} is attr.", attr.clone().unwrap().as_str());
        attr.is_some() && attr.unwrap() == "*"
    });
    assert!(out.is_some());
}

#[test]
fn test_get_name() {
    let this_name = "let x = proceed_onetwothree(one, two, three)";
    let proceed = SynHelper::get_proceed(this_name.to_string());
    assert_eq!(proceed, "_onetwothree");
    let this_name = "let x = proceed___onetwothree()";
    let proceed = SynHelper::get_proceed(this_name.to_string());
    assert_eq!(proceed, "___onetwothree");
}

#[test]
fn test_knockoff_factories() {
    assert!(get_knockoff_factores_arg().is_some());
}

fn get_knockoff_factores_arg() -> Option<String> {
    env::var("KNOCKOFF_FACTORIES").ok()
}

fn do_test(attr_matcher: &dyn Fn(&Attribute) -> bool) -> Option<ItemFn> {
    let option = get_knockoff_factores_arg();
    let mut did_check = false;
    assert!(option.is_some());
    option.map(|aug_file| {
        let p = Path::new(&aug_file);
        if p.exists() {
            let mut f = File::open(p).unwrap();
            let f = SynHelper::parse_syn_file(&mut f);
            let out = f.unwrap().items.iter().flat_map(|f| {
                match f {
                    Item::Fn(item_fn) => {
                        if item_fn.sig.ident.to_string().as_str() == "do_aspect" {
                            return vec![item_fn];
                        }
                        vec![]
                    }
                    _ => {
                        vec![]
                    }
                }
            })
                .map(|item_fn| {
                    let attribute = item_fn.attrs[0].clone();
                    assert!(attr_matcher(&attribute));
                    did_check = true;
                    Some(item_fn.clone())
                }).next().flatten();
            return out
        }
        None::<ItemFn>
    }).flatten().map(|f| {
        assert!(did_check);
        f
    })
}