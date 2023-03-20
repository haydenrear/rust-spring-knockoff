use crate::parse_container::ParseContainer;

pub trait BuildParseContainer {
    fn build_parse_container(&self, parse_container: &mut ParseContainer);
}



