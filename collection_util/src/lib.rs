use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::marker::PhantomData;

pub fn add_to_multi_value<K: Hash + Eq, V>(
    multi_value_map: &mut HashMap<K, Vec<V>>, to_add: V, to_add_k: K) {
    if !multi_value_map.contains_key(&to_add_k) {
        multi_value_map.insert(to_add_k, vec![to_add]);
    } else {
        multi_value_map.get_mut(&to_add_k).unwrap().push(to_add);
    }
}

pub fn add_or_insert<T, U>(
    key_value: &T,
    group_value: U,
    mut values: &mut HashMap<T, HashSet<U>>
)
    where
        T: Hash + Eq + Clone,
        U: Eq + Hash
{
    if values.get(key_value).is_none() {
        values.insert(key_value.clone(), HashSet::from([group_value]));
    } else {
        if values.get(key_value).filter(|c| c.contains(&group_value)).is_none() {
            values.get_mut(key_value)
                .map(|indices| indices.insert(group_value));
        }
    }
}

pub fn group_by_key<K, V>(map: Vec<(K, V)>) -> HashMap<K, HashSet<V>>
    where
        K: Eq + Hash,
        V: Clone + Hash + Eq
{
    let mut result: HashMap<K, HashSet<V>> = HashMap::new();
    for (key, value) in map.into_iter() {
        if result.contains_key(&key) {
            result.get_mut(&key)
                .map(|f| {
                    f.insert(value);
                });
        } else {
            let mut v = HashSet::new();
            v.insert(value);
            result.insert(key, v);
        }
    }
    result
}



pub struct MultiMap<K, V, IT>(pub HashMap<K, Vec<V>>, PhantomData<IT>)
where
    IT: Iterator<Item=(K, V)>,
    K:  Hash + Eq + PartialEq + Ord;


impl<K, V, IT> MultiMap<K, V, IT>
    where
        K: Hash + Eq + PartialEq + Ord,
        V: Hash + Eq + Ord,
        IT: Iterator<Item=(K, V)>
{
    fn collect(to_collect: IT) -> MultiMap<K, V, IT> {
        let mut out_map = HashMap::new();
        to_collect.for_each(|(k, v)| {
            add_to_multi_value(&mut out_map, v, k);
        });
        MultiMap(out_map, PhantomData::default())
    }
}

pub trait IntoMultiMap {
    type K: Hash + Eq + PartialEq + Ord;
    type V;
    fn collect_multi(&mut self)  -> MultiMap<Self::K, Self::V, Self> where Self: Sized + Iterator<Item=(Self::K, Self::V)> {
        let mut out = HashMap::new();
        self.for_each(|(k, v)| {
            add_to_multi_value(&mut out, v, k) ;
        });
        MultiMap(out, PhantomData::default())
    }
}

impl<K, V, INT> IntoMultiMap for INT
    where INT: Iterator<Item=(K, V)>,
          K: Hash + Eq + PartialEq + Ord
{
    type K = K;
    type V = V;
}