#![feature(str_split_remainder)]
#![feature(async_fn_in_trait)]
#[macro_use]
extern crate alloc;
extern crate core;

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::ops::Deref;
use std::sync::{Arc, Mutex, MutexGuard};


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
    pub mod context_builder;
}



pub struct Gen<T: ?Sized>{
    inner: Arc<T>

}
pub struct Gen2<T: ?Sized>{
    inner: Arc<T>,
    phantom: PhantomGuy<T>
}

pub struct Gen2Mutex<T: ?Sized>{
    pub inner: Arc<Mutex<T>>,
    pub phantom: PhantomGuy<T>
}


pub struct PhantomGuy<T: ?Sized> {
    phantom: PhantomData<T>
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

impl <T: 'static + Send + Sync> Gen2<T> {
    fn to_any(self) -> Gen2<dyn Any + Send + Sync> {
        let found = self.inner.clone() as Arc<dyn Any + Send + Sync>;
        let phantom = PhantomGuy{
            phantom: PhantomData::default()
        };

        Gen2 {
            inner: found,
            phantom: phantom as PhantomGuy<dyn Any + Send + Sync>
        }
    }

    pub fn downcast_ref_gen(self) -> Arc<T>{
        self.to_any().inner.clone().downcast().unwrap()
    }
}

pub trait SMutex {
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

    let gen_2 = Gen2 {
        inner: Arc::new((One{})),
        phantom: PhantomGuy { phantom: Default::default() },
    };
    let new_any = gen_2.to_any();

    let mutex = Arc::new(Mutex::new(One {}));
    let mutex = mutex.clone() as Arc<dyn Any + Send + Sync + 'static>;
    let m = mutex.downcast::<Mutex<One>>().unwrap();
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