use self::sealed::Sealed;

/// A helper trait enforcing that the type is `Option`.
pub trait IsOption: Sealed {
    /// The type of inner value.
    type Item;

    /// Consume itself and get the value of `Option`.
    fn into_option(self) -> Option<Self::Item>;
}

impl<T> IsOption for Option<T> {
    type Item = T;

    #[inline(always)]
    fn into_option(self) -> Option<Self::Item> {
        self
    }
}

mod sealed {
    pub trait Sealed {}

    impl<T> Sealed for Option<T> {}
}
