use std::iter::{Fuse, Map};
struct FlattenCompat<I> {
    iter: Option<I>
}

impl<I> FlattenCompat<I>
{
    /// Adapts an iterator by flattening it, for use in `flatten()` and `flat_map()`.
    fn new(iter: Option<I>) -> FlattenCompat<I> {
        FlattenCompat { iter }
    }
}

pub struct FlatMapOption<I> {
    inner: Option<I>,
}

pub trait FlatMapOptional {
    type Item;
    fn flat_map<U, F>(self, f: F) -> Option<U>
        where
            Self: Sized,
            F: FnMut(Option<Self::Item>) -> Option<U>;

}


impl <I> FlatMapOptional for I {
    type Item = I;

    fn flat_map<U, F>(self, mut f: F) -> Option<U>
        where
            Self: Sized,
            F: FnMut(Option<Self::Item>) -> Option<U>
    {
        let option = f(Some(self));
        option
    }
}

#[test]
fn test() {
    let out = Some("ok");
    let out = out.flat_map(|o| Some("whatever"));
    assert_eq!(out.unwrap(), "whatever");
}

