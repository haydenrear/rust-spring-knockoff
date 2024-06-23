use syn::{Path, PathSegment};
use codegen_utils::FlatMapOptional;

pub trait KnockoffEquals<T> {
    fn k_eq(&self, t: &T) -> bool;
}


impl KnockoffEquals<PathSegment> for PathSegment {
    fn k_eq(&self, t: &PathSegment) -> bool {
        t.ident.to_string() == self.ident.to_string()
    }
}

impl KnockoffEquals<Path> for Path {
    fn k_eq(&self, t: &Path) -> bool {
        t.segments.iter().all(|p| self.segments.iter().any(|p_s| p_s.k_eq(&p)))
    }
}

impl <T> KnockoffEquals<Option<&T>> for Option<&T>
where
    T: KnockoffEquals<T>
{
    fn k_eq(&self, t: &Option<&T>) -> bool {
        if self.as_ref().is_none() && t.as_ref().is_some() {
            false
        } else if t.as_ref().is_none() && self.as_ref().is_some() {
            false
        } else if t.as_ref().is_none() && self.as_ref().is_none() {
            true
        } else {
            t.as_ref()
                .flat_map_opt(|u| self.as_ref().map(|t| u.k_eq(t)))
                .unwrap()
        }
    }
}

impl <T> KnockoffEquals<&T> for T
where T: KnockoffEquals<T> {
    fn k_eq(&self, t: &&T) -> bool {
        let t: &T = *t;
        self.k_eq(t)
    }
}

#[test]
fn test_if() {
    pub struct TestKnockoffEq {
        one: String
    }
    impl KnockoffEquals<TestKnockoffEq> for TestKnockoffEq {
        fn k_eq(&self, t: &TestKnockoffEq) -> bool {
            t.one == self.one
        }
    }

    let one = Some(TestKnockoffEq {one: "hello".to_string()});
    let two = Some(TestKnockoffEq {one: "hello".to_string()});
    let one_no = Some(TestKnockoffEq {one: "hello".to_string()});
    let two_no = Some(TestKnockoffEq {one: "goodbye".to_string()});

    assert!(one.as_ref().k_eq(&two.as_ref()));
    assert!(!one_no.as_ref().k_eq(&two_no.as_ref()));
    assert!(&one.as_ref().k_eq(&&two.as_ref()));
    assert!(!&one_no.as_ref().k_eq(&&two_no.as_ref()));
}