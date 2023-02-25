pub trait Converter<From, To> {
    fn convert(&self, from: &From) -> To;
}
