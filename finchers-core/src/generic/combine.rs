use super::hlist::{HCons, HList, Tuple};

pub trait CombineBase<T: HList> {
    type Out: HList;

    fn combine(self, other: T) -> Self::Out;
}

// forall T:Hlist => (T, ()) -> T
impl<T: HList> CombineBase<T> for () {
    type Out = T;

    fn combine(self, other: T) -> Self::Out {
        other
    }
}

impl<H, T: HList, U: HList> CombineBase<U> for HCons<H, T>
where
    T: CombineBase<U>,
    HCons<H, <T as CombineBase<U>>::Out>: HList,
{
    type Out = HCons<H, <T as CombineBase<U>>::Out>;

    #[inline(always)]
    fn combine(self, other: U) -> Self::Out {
        HCons {
            head: self.head,
            tail: self.tail.combine(other),
        }
    }
}

pub trait Combine<T: Tuple>: Tuple + sealed::Sealed<T> {
    type Out: Tuple;

    fn combine(self, other: T) -> Self::Out;
}

impl<H: Tuple, T: Tuple> Combine<T> for H
where
    H::HList: CombineBase<T::HList>,
{
    type Out = <<H::HList as CombineBase<T::HList>>::Out as HList>::Tuple;

    fn combine(self, other: T) -> Self::Out {
        self.hlist().combine(other.hlist()).tuple()
    }
}

mod sealed {
    use super::{CombineBase, Tuple};

    pub trait Sealed<T> {}

    impl<H: Tuple, T: Tuple> Sealed<T> for H where H::HList: CombineBase<T::HList> {}
}

#[cfg(test)]
mod tests {
    use super::*;

    fn combine<H: Tuple, T: Tuple>(h: H, t: T) -> H::Out
    where
        H: Combine<T>,
    {
        h.combine(t)
    }

    #[test]
    fn case1_units() {
        let a = ();
        let b = ();
        assert_eq!(combine(a, b), ());
    }

    #[test]
    fn case2_unit1() {
        let a = (10,);
        let b = ();
        assert_eq!(combine(a, b), (10,));
    }

    #[test]
    fn case3_unit2() {
        let a = ();
        let b = (10,);
        assert_eq!(combine(a, b), (10,));
    }

    #[test]
    fn case4_complicated() {
        let a = ("a", "b", "c");
        let b = (10, 20, 30);
        assert_eq!(combine(a, b), ("a", "b", "c", 10, 20, 30));
    }

    #[test]
    fn case5_nested() {
        let a = ("a", ("b", "c"));
        let b = (10, (20,), 30);
        assert_eq!(combine(a, b), ("a", ("b", "c"), 10, (20,), 30));
    }
}
