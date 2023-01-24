#[macro_use]
extern crate alloc;

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;


pub mod web_framework {
    pub mod context;
    pub mod dispatch;
    pub mod convert;
    pub mod filter;
    pub mod http;
    pub mod message;
    pub mod request;
    pub mod security;
    pub mod session;
}

pub struct Gen<T: ?Sized>{
    inner: Arc<T>
}

pub struct Two {

}

pub struct One {

}

impl <T: 'static> Gen<T> {
    fn to_any(&mut self) -> Gen<dyn Any> {
        let found = self.inner.clone() as Arc<dyn Any>;
        Gen {
            inner: found
        }
    }
}

fn contains<T: 'static>(type_id: TypeId) -> bool {
    TypeId::of::<T>() == type_id
}

#[test]
fn test_downcast() {
    let d = add_to();
    assert_ne!(d.len(), 0);
    let mut one_gen = Gen {
        inner: Arc::new(One{})
    };
    let two_gen = Gen {
        inner: Arc::new(Two{})
    };
    let found = d.get(&one_gen.type_id().clone()).unwrap();
    let found_dc = found.inner.downcast_ref::<One>();
    assert!(found_dc.is_some());
    let found = d.get(&two_gen.type_id().clone()).unwrap();
    let found_dc = found.inner.downcast_ref::<Two>();
    assert!(found_dc.is_some());
}

fn add_to<'a>() -> HashMap<TypeId, Gen<dyn Any>> {
    let mut one_gen = Gen {
        inner: Arc::new(One{})
    };
    let mut two_gen = Gen {
        inner: Arc::new(Two{})
    };

    let mut map: HashMap<TypeId, Gen<dyn Any>> = HashMap::new();

    map.insert(one_gen.type_id().clone(), one_gen.to_any());
    map.insert(two_gen.type_id().clone(), two_gen.to_any());
    map
}