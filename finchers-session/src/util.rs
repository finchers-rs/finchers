pub(crate) trait BuilderExt: Sized {
    fn if_some<T>(self, value: Option<T>, f: impl FnOnce(Self, T) -> Self) -> Self {
        if let Some(value) = value {
            f(self, value)
        } else {
            self
        }
    }
}

impl<T> BuilderExt for T {}
