use std::collections::HashMap;
use crate::bean::BeanDefinition;
use crate::profile_tree::ProfileTree;

pub trait ProfileTreeModifier {
    fn modify_bean(&self, dep_type: &mut BeanDefinition, profile_tree: &mut ProfileTree);
    fn new(profile_tree_items: &HashMap<String, BeanDefinition>) -> Self
    where Self: Sized;

}
