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

pub struct MapContainer {
    values: HashMap<TypeId, Gen<dyn Any + Send + Sync>>
}

impl MapContainer {
    fn get_values(&self) -> Option<Arc<dyn Any + Send + Sync>> {
        for key_val in self.values.iter() {
            return Some(key_val.1.inner.clone());
        }
        None
    }
}

pub struct Two {

}

pub struct One {
}

impl <T: 'static + Send + Sync> Gen<T> {
    fn to_any(self) -> Gen<dyn Any + Send + Sync> {
        let found = self.inner.clone() as Arc<dyn Any + Send + Sync>;
        Gen {
            inner: found
        }
    }

    pub fn downcast_ref_gen(self) -> Arc<T>{
        self.to_any().inner.clone().downcast().unwrap()
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
    let three_gen = Gen {
        inner: Arc::new(Two{})
    };
    let four_gen = Gen {
        inner: Arc::new(Two{})
    };
    let ref_gen = four_gen.downcast_ref_gen();
    let found = d.get(&one_gen.type_id().clone()).unwrap();
    let found_dc = found.inner.downcast_ref::<One>();
    assert!(found_dc.is_some());
    let found = d.get(&two_gen.type_id().clone()).unwrap();
    let found_dc = found.inner.downcast_ref::<Two>();
    assert!(found_dc.is_some());

    let f =add_to();
    let mc  = MapContainer {
        values: f
    };
    let opt = mc.get_values();

    assert_eq!(&three_gen.type_id(), &two_gen.type_id());

    let three_rc = Arc::new(three_gen);
    assert_eq!(three_rc.deref().type_id(), two_gen.type_id());

    let gen = one_gen.to_any();
    let f  = gen
        .inner.downcast::<One>();

    assert!(f.is_ok());
}

fn add_to<'a>() -> HashMap<TypeId, Gen<dyn Any + Send + Sync>> {
    let mut one_gen = Gen {
        inner: Arc::new(One{})
    };
    let mut two_gen = Gen {
        inner: Arc::new(Two{})
    };

    let mut map: HashMap<TypeId, Gen<dyn Any + Send + Sync>> = HashMap::new();

    map.insert(one_gen.type_id().clone(), one_gen.to_any());
    map.insert(two_gen.type_id().clone(), two_gen.to_any());
    map
}