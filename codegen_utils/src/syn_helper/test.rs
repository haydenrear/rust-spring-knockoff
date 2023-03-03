use std::env;
use std::fs::File;
use std::path::Path;
use syn::{Item, ItemFn};
use crate::parse::parse_syn_file;
use crate::syn_helper::SynHelper;

#[test]
fn test_syn_helper() {
    let mut did_check = false;
    let out = env::var("KNOCKOFF_FACTORIES").map(|aug_file| {
        let p = Path::new(&aug_file);
        if p.exists() {
            let mut f = File::open(p).unwrap();
            let f = parse_syn_file(&mut f);
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
                let attr = SynHelper::parse_attr_path_single(&item_fn.attrs[0].clone());
                assert!(attr.is_some());
                assert_eq!(attr.unwrap(), "*");
                did_check = true;
                Some(item_fn.clone())
            }).next().flatten();
            return out
        }
        None::<ItemFn>
    });
    assert!(out.is_ok());
    assert!(did_check);
}