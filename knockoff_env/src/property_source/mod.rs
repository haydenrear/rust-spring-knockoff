use std::collections::HashMap;
use std::fmt::Debug;
use std::io::ErrorKind;
use std::str::FromStr;
use serde::de::Error;
use serde::{Deserialize, Serialize};

use knockoff_logging::*;
use lazy_static::lazy_static;
use std::sync::Mutex;
use knockoff_resource::FileResource;
use crate::{EnvironmentProfiles, EnvProfile, logger_lazy};
import_logger!("property_source.rs");

pub struct GetProperty {
    property: HashMap<String, String>
}

impl GetProperty {
    pub fn new(property: HashMap<String,String>) -> Self{
        Self {property}
    }
}

impl GetProperty {
    fn get_property_as<T>(&self, name: &str) -> Result<T, std::io::Error>
    where
        T: for<'a> Deserialize<'a>
    {
        if !self.property.contains_key(name) {
            Err(std::io::Error::new::<serde_json::Error>(ErrorKind::NotFound,
                                    Error::custom("Property source did not contain key.")))
        } else {
            let prop = self.property.get(name).unwrap();
            serde_json::from_str::<T>(prop)
                .map_err(|serde_json_err| std::io::Error::new::<serde_json::Error>(ErrorKind::InvalidData,
                         Error::custom("Property source did not contain key.")))
        }
    }
}

fn parse_property_as<T: FromStr>(property: &str) -> Option<T> {
    property.parse().map_err(|e| {
        error!("Error parsing property named: {}", property);
    }).ok()
}


pub trait PropertySource<T> {
    fn parse_property<U: FromStr>(&self, property_name: &str) -> Option<U> {
        self.get_property_as_str(property_name)
            .map(|prop| parse_property_as(prop))
            .flatten()
    }
    fn contains_property(&self, property_name: &str) -> bool;
    fn get_property_as_str(&self, property_name: &str) -> Option<&str>;
    fn get_property_as_bool(&self, property_name: &str) -> Option<bool> {
        self.parse_property(property_name)
    }
    fn get_property_as_string(&self, property_name: &str) -> Option<String> {
        self.get_property_as_str(property_name)
            .map(|prop| prop.to_string())
    }
    fn get_property_as_int(&self, property_name: &str) -> Option<i64> {
        self.parse_property(property_name)
    }
    fn get_property_source_name(&self) -> &str;
    fn properties(&self) -> &T;
}

pub trait GetPropertyPropertySource: PropertySource<GetProperty> {
    fn get_property_as<T>(&self, property_name: &str) -> Result<T, std::io::Error>
        where T: for<'a> Deserialize<'a>
    {
        self.properties().get_property_as(property_name)
    }
}

pub struct TomlPropertySource(GetProperty, EnvProfile, String);

impl TomlPropertySource {
    pub fn new(file: FileResource) -> Self {
        todo!()
    }
}

impl PropertySource<GetProperty> for TomlPropertySource {
    fn contains_property(&self, property_name: &str) -> bool {
        self.0.property.contains_key(property_name)
    }

    fn get_property_as_str(&self, property_name: &str) -> Option<&str> {
        let prop = self.0.property.get(property_name);
        if prop.is_some() {
            Some(prop.unwrap().as_str())
        } else {
            None
        }
    }

    fn get_property_source_name(&self) -> &str {
        self.2.as_str()
    }

    fn properties(&self) -> &GetProperty {
        &self.0
    }
}

impl GetPropertyPropertySource for TomlPropertySource {
}

#[test]
fn test_get_property() {
    pub struct TestGetProperty(GetProperty);
    impl PropertySource<GetProperty> for TestGetProperty {
        fn contains_property(&self, property_name: &str) -> bool {
            todo!()
        }

        fn get_property_as_str(&self, property_name: &str) -> Option<&str> {
            todo!()
        }

        fn get_property_source_name(&self) -> &str {
            todo!()
        }

        fn properties(&self) -> &GetProperty {
            &self.0
        }
    }

    impl GetPropertyPropertySource for TestGetProperty {
    }

    #[derive(Serialize, Deserialize)]
    pub struct Prop;

    let mut props = HashMap::new();
    props.insert("test".to_string(), serde_json::to_string(&Prop).unwrap().to_string());
    let test_prop = TestGetProperty(GetProperty::new(props));
    let prop = test_prop.get_property_as::<Prop>("test");

    assert!(prop.is_ok());
}