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

pub trait FlatMapResult<I, E> {
    fn flat_map_res<U, F>(self, f: F) -> Result<U, E>
        where
            Self: Sized,
            F: FnMut(I) -> Result<U, E>;

}


impl <I, E: std::error::Error> FlatMapResult<I, E> for Result<I, E> {
    fn flat_map_res<U, F>(self, mut f: F) -> Result<U, E>
        where
            Self: Sized,
            F: FnMut(I) -> Result<U, E>{
        let r = self.map(f);
        if r.is_ok() {
            r.unwrap()
        } else {
            Err(r.err().unwrap())
        }
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

#[test]
fn test_res() {
    let out = Ok("ok");
    let out = out.flat_map_res(|o| Ok("whatever"));
    assert_eq!(out.unwrap(), "whatever");
    let out: Result<&str, &str> = out.flat_map_res(|o| Err("Err"));
    assert!(out.is_err());
    assert_eq!(out.err(), Some("Err"));
}
