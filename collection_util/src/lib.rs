use std::collections::HashMap;
use std::hash::Hash;

pub fn add_to_multi_value<K: Hash + Eq + PartialOrd + PartialOrd + Ord, V>(
    multi_value_map: &mut HashMap<K, Vec<V>>, to_add: V, to_add_k: K) {
    if !multi_value_map.contains_key(&to_add_k) {
        multi_value_map.insert(to_add_k, vec![to_add]);
    } else {
        multi_value_map.get_mut(&to_add_k).unwrap().push(to_add);
    }
}