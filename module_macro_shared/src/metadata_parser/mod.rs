use std::sync::Arc;
use serde::{Deserialize, Serialize};
use crate::parse_container::{MetadataItem, ParseContainer};

pub trait ParseMetadataItem<T: MetadataItem> {
    fn parse_values(parse_container: &mut Option<Box<dyn MetadataItem>>) -> Option<&mut T>;
}

#[macro_export]
macro_rules! impl_parse_values {
    ($ty:ty) => {
        impl $ty {
            pub fn parse_values(mut v: &mut Option<Box<dyn MetadataItem>>) -> Option<&mut Self> {
                if v.is_none() {
                    return None;
                }
                let mut v = v.as_mut().unwrap();
                let out = v.as_any().downcast_mut::<$ty>();
                if out.is_some()  {
                    Some(out.unwrap())
                } else {
                    None
                }
            }
        }
    }
}