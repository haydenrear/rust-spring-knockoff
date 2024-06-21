
#[aspect(**)]
#[ordered(0)]
pub fn do_aspect(&self, one: One) -> String {
    println!("hello");
    println!("{}", self.two.clone());
    let found = self.proceed_one_test(one);
    found
}

#[aspect(**)]
#[ordered(1)]
pub fn second_aspect(&self, one: One) -> String {
    println!("another_hello");
    println!("{}", self.two.clone());
    let found = self.proceed_one_test_again(one);
    "another".to_string()
}

pub struct Test;

impl Test {
    fn do_something() {
        println!("hello")
    }
}

fn do_something_test() {
    println!("hello")
}