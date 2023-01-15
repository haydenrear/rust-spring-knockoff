use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

pub struct Types {
    pub values: Mutex<HashMap<String, String>>,
}

lazy_static! {
    pub static ref types: Arc<Types> = Arc::new(Types {
        values: Mutex::new(HashMap::new())
    });
}

#[macro_export]
macro_rules! last_thing {
    ($ident:ident) => {
        let x = types.values.lock().unwrap().get(&String::from("hello"));
        println!("x");
    };
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
