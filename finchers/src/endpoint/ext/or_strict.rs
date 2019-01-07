use {
    crate::{
        endpoint::{
            ActionContext, //
            Endpoint,
            EndpointAction,
            IsEndpoint,
            Preflight,
            PreflightContext,
        },
        error::Error,
    },
    futures::Poll,
    http::StatusCode,
};

#[allow(missing_docs)]
#[derive(Debug, Copy, Clone)]
pub struct OrStrict<E1, E2> {
    pub(super) e1: E1,
    pub(super) e2: E2,
}

impl<E1: IsEndpoint, E2: IsEndpoint> IsEndpoint for OrStrict<E1, E2> {}

impl<E1, E2, Bd> Endpoint<Bd> for OrStrict<E1, E2>
where
    E1: Endpoint<Bd>,
    E2: Endpoint<Bd, Output = E1::Output>,
{
    type Output = E1::Output;
    type Action = OrStrictAction<E1::Action, E2::Action>;

    fn action(&self) -> Self::Action {
        OrStrictAction {
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
pub struct OrStrictAction<L, R> {
    state: State<L, R>,
}

impl<L, R, Bd> EndpointAction<Bd> for OrStrictAction<L, R>
where
    L: EndpointAction<Bd>,
    R: EndpointAction<Bd, Output = L::Output>,
{
    type Output = L::Output;

    fn preflight(
        &mut self,
        cx: &mut PreflightContext<'_>,
    ) -> Result<Preflight<Self::Output>, Error> {
        self.state = match std::mem::replace(&mut self.state, State::Done) {
            State::Init(mut left, mut right) => {
                let orig_cx = cx.clone();
                match left.preflight(cx) {
                    Ok(Preflight::Incomplete) => State::Left(left),
                    Ok(Preflight::Completed(output)) => return Ok(Preflight::Completed(output)),
                    Err(e1) => {
                        *cx = orig_cx;
                        match right.preflight(cx) {
                            Ok(Preflight::Incomplete) => State::Right(right),
                            Ok(Preflight::Completed(output)) => {
                                return Ok(Preflight::Completed(output))
                            }
                            Err(e2) => {
                                return Err(match (e1.status_code(), e2.status_code()) {
                                    (_, StatusCode::NOT_FOUND)
                                    | (_, StatusCode::METHOD_NOT_ALLOWED) => e1,
                                    (StatusCode::NOT_FOUND, _)
                                    | (StatusCode::METHOD_NOT_ALLOWED, _) => e2,
                                    (status1, status2) if status1 >= status2 => e1,
                                    _ => e2,
                                });
                            }
                        }
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
            State::Init(..) => panic!(),
            State::Left(ref mut t) => t.poll_action(cx).map_err(Into::into),
            State::Right(ref mut t) => t.poll_action(cx).map_err(Into::into),
            State::Done => panic!(),
        }
    }
}
