use std::any::Any;
use std::fmt::{Debug, Formatter};
use std::fs::Metadata;
use std::ops::Deref;
use std::rc::Rc;
use std::sync::Arc;
use syn::Item;
use module_macro_shared::parse_container::MetadataItem;
use crate::aspect_knockoff_provider::aspect_item_modifier::AspectParser;
use crate::aspect_knockoff_provider::aspect_parse_provider::MethodAdviceAspectCodegen;

#[test]
fn test_parse_aspect() {
    // let mut aspects = AspectParser::parse_aspects();
    // assert_eq!(aspects.aspects.len(), 1);
    // let mut first = aspects.aspects.remove(0);
    // assert_eq!(first.method_advice_aspects.len(), 1);
    // let method_advice = first.method_advice_aspects.remove(0);
    // assert!(!method_advice.before_advice.clone().unwrap().to_token_stream().to_string().as_str().contains("proceed"));
    // assert!(!method_advice.after_advice.clone().unwrap().to_token_stream().to_string().as_str().contains("proceed"));
    // assert!(method_advice.item.is_some());
    // match method_advice.item.unwrap() {
    //     Item::Fn(fn_found) => {
    //         assert_eq!(fn_found.sig.ident.to_string().clone(), "do_aspect");
    //     }
    //     _ => {
    //         assert!(false)
    //     }
    // }
}

#[derive(Clone)]
pub struct Test {

}


impl Debug for Test {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}
pub trait TestMetadataItem: 'static + Send + Sync {
    fn as_any(&mut self) -> &mut dyn Any;
}

impl TestMetadataItem for Test {
    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
}


#[test]
fn test_dyn_values(){
    let mut out = Box::new(Test {});
    let mut out = out.as_mut().as_any().downcast_mut::<Test>().unwrap();
    // let out = out.downcast_mut::<Test>().unwrap();

}