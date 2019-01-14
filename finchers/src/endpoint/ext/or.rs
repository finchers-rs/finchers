use {
    super::NotMatched,
    crate::{
        action::{
            ActionContext, //
            EndpointAction,
            Preflight,
            PreflightContext,
        },
        endpoint::{Endpoint, IsEndpoint},
        error::Error,
    },
    either::Either,
    futures::Poll,
};

#[allow(missing_docs)]
#[derive(Debug, Copy, Clone)]
pub struct Or<E1, E2> {
    pub(super) e1: E1,
    pub(super) e2: E2,
}

impl<E1: IsEndpoint, E2: IsEndpoint> IsEndpoint for Or<E1, E2> {}

impl<E1, E2, T1, T2, Bd> Endpoint<Bd> for Or<E1, E2>
where
    E1: Endpoint<Bd, Output = (T1,)>,
    E2: Endpoint<Bd, Output = (T2,)>,
{
    type Output = (Either<T1, T2>,);
    type Action = OrAction<E1::Action, E2::Action>;

    fn action(&self) -> Self::Action {
        OrAction {
            state: State::Init(self.e1.action(), self.e2.action()),
        }
    }
}

#[allow(missing_debug_implementations)]
enum State<L, R> {
    Init(L, R),
    Left(L),
    Right(R),
    Done,
}

#[allow(missing_debug_implementations)]
pub struct OrAction<L, R> {
    state: State<L, R>,
}

impl<E1, E2, T1, T2, Bd> EndpointAction<Bd> for OrAction<E1, E2>
where
    E1: EndpointAction<Bd, Output = (T1,)>,
    E2: EndpointAction<Bd, Output = (T2,)>,
{
    type Output = (Either<T1, T2>,);

    fn preflight(
        &mut self,
        cx: &mut PreflightContext<'_>,
    ) -> Result<Preflight<Self::Output>, Error> {
        self.state = match std::mem::replace(&mut self.state, State::Done) {
            State::Init(mut left, mut right) => {
                let orig_cx = cx.clone();
                let left_output = left.preflight(cx);
                let cx1 = std::mem::replace(cx, orig_cx);
                let right_output = right.preflight(cx);

                match (left_output, right_output) {
                    (Ok(l), Ok(r)) => {
                        // If both endpoints are matched, the one with the larger number of
                        // (consumed) path segments is choosen.
                        if cx1.num_popped_segments() >= cx.num_popped_segments() {
                            *cx = cx1;
                            if let Preflight::Completed((output,)) = l {
                                return Ok(Preflight::Completed((Either::Left(output),)));
                            } else {
                                State::Left(left)
                            }
                        } else if let Preflight::Completed((output,)) = r {
                            return Ok(Preflight::Completed((Either::Right(output),)));
                        } else {
                            State::Right(right)
                        }
                    }

                    (Ok(l), Err(..)) => {
                        *cx = cx1;
                        if let Preflight::Completed((output,)) = l {
                            return Ok(Preflight::Completed((Either::Left(output),)));
                        } else {
                            State::Left(left)
                        }
                    }

                    (Err(..), Ok(r)) => {
                        if let Preflight::Completed((output,)) = r {
                            return Ok(Preflight::Completed((Either::Right(output),)));
                        } else {
                            State::Right(right)
                        }
                    }

                    (Err(left), Err(right)) => {
                        return Err(NotMatched {
                            left,
                            right,
                            _priv: (),
                        }
                        .into())
                    }
                }
            }
            _ => panic!("unexpected condition"),
        };

        Ok(Preflight::Incomplete)
    }

    #[inline]
    fn poll_action(&mut self, cx: &mut ActionContext<'_, Bd>) -> Poll<Self::Output, Error> {
        match self.state {
            State::Left(ref mut t) => t
                .poll_action(cx)
                .map(|x| x.map(|(out,)| (Either::Left(out),)))
                .map_err(Into::into),
            State::Right(ref mut t) => t
                .poll_action(cx)
                .map(|x| x.map(|(out,)| (Either::Right(out),)))
                .map_err(Into::into),
            _ => panic!("unexpected condition"),
        }
    }
}
