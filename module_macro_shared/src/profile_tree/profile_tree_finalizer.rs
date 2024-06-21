use crate::DefaultBuildParseContainer;
use crate::parse_container::ParseContainer;
use crate::profile_tree::ProfileTree;

pub trait ProfileTreeFinalizer {
    fn finalize(parse_container: &mut ParseContainer);
}

#[derive(Default)]
pub struct DefaultProfileTreeFinalizer;
impl ProfileTreeFinalizer for DefaultProfileTreeFinalizer { fn finalize(parse_container: &mut ParseContainer) {} }