pub mod wait_for {
    pub mod wait_async;
    pub mod test;
}

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use crate::wait_for::test::One;
    use crate::wait_for::wait_async::WaitFor;
    use super::*;

    #[test]
    fn it_works() {
        let one = One;
        let wait_for = WaitFor;
    }
}
