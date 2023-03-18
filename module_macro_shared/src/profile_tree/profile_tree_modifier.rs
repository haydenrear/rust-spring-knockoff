use std::collections::HashMap;
use crate::bean::Bean;
use crate::profile_tree::ProfileTree;

pub trait ProfileTreeModifier {
    fn modify_bean(&self, dep_type: &mut Bean, profile_tree: &mut ProfileTree);
    fn new(profile_tree_items: &HashMap<String,Bean>) -> Self
    where Self: Sized;
}
