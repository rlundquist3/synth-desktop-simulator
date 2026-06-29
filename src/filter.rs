use rodio::Source;

#[derive(Debug)]
pub struct Filter<T: Source<Item = f32>> {
    input: T,
    on: bool,
}

impl<T> Iterator for Filter<T>
where
    T: Source,
{
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        self.input.next()
    }
}
