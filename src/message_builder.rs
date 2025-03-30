#[derive(Debug)]
pub struct MessageBuilder<T> {
    pub inner: T,
}

impl<T> MessageBuilder<T>
where
    T: Default + MessageBuilderOperation,
{
    pub fn new() -> Self {
        Self {
            inner: T::default(),
        }
    }

    pub fn build(self) -> anyhow::Result<T> {
        self.inner.finalize()
    }
}

pub trait MessageBuilderOperation: Sized {
    fn finalize(self) -> anyhow::Result<Self>;
}
