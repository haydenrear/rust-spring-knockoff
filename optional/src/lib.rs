pub trait FlatMapOptional<I> {
    fn flat_map_opt<U, F>(self, f: F) -> Option<U>
        where
            Self: Sized,
            F: FnMut(I) -> Option<U>;

}


impl <I> FlatMapOptional<I> for Option<I> {
    fn flat_map_opt<U, F>(self, mut f: F) -> Option<U>
        where
            Self: Sized,
            F: FnMut(I) -> Option<U>
    {
        self.map(f).flatten()
    }
}

#[test]
fn test() {
    let out = Some("ok");
    let out = out.flat_map_opt(|o| Some("whatever"));
    assert_eq!(out.unwrap(), "whatever");
    let out: Option<&str> = out.flat_map_opt(|o| None);
    assert!(out.is_none());
}

