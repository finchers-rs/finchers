use self::sealed::Sealed;

/// A helper trait enforcing that the type is a "Option".
pub trait IsOption: Sealed {
    /// The type of inner value.
    type Item;

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
