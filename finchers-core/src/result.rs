use self::sealed::Sealed;

/// A helper trait enforcing that the type is `Result`.
pub trait IsResult: Sealed {
    /// The type of success value.
    type Ok;

    /// The type of error value.
    type Err;

    /// Consume itself and get the value of `Result`.
    fn into_result(self) -> Result<Self::Ok, Self::Err>;
}

impl<T, E> IsResult for Result<T, E> {
    type Ok = T;
    type Err = E;

    #[inline(always)]
    fn into_result(self) -> Result<Self::Ok, Self::Err> {
        self
    }
}

mod sealed {
    pub trait Sealed {}

    impl<T, E> Sealed for Result<T, E> {}
}
