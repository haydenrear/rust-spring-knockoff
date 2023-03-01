use syn::{Item, Path};

pub trait Aspect {
    fn does_match(item: Item, package: AspectAwarePackage) -> bool;
}

pub struct AspectAwarePackage {
    module_name: String,
    path: Path
}