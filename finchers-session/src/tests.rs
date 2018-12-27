use finchers::error::Error;
use finchers::input::Input;
use finchers::prelude::*;
use finchers::test;

use futures::future;
use http::Request;

use session::{RawSession, Session};

use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq)]
enum Op {
    Get,
    Set(String),
    Remove,
    Write,
}

#[derive(Default)]
struct CallChain {
    chain: RefCell<Vec<Op>>,
}

impl CallChain {
    fn register(&self, op: Op) {
        self.chain.borrow_mut().push(op);
    }

    fn result(&self) -> Vec<Op> {
        self.chain.borrow().clone()
    }
}

struct MockSession {
    call_chain: Rc<CallChain>,
}

impl RawSession for MockSession {
    type WriteFuture = future::FutureResult<(), Error>;

    fn get(&self) -> Option<&str> {
        self.call_chain.register(Op::Get);
        None
    }

    fn set(&mut self, value: String) {
        self.call_chain.register(Op::Set(value));
    }

    fn remove(&mut self) {
        self.call_chain.register(Op::Remove);
    }

    fn write(self, _: &mut Input) -> Self::WriteFuture {
        self.call_chain.register(Op::Write);
        future::ok(())
    }
}

#[test]
fn test_session_with() {
    let call_chain = Rc::new(CallChain::default());

    let mut runner = test::runner({
        let session_endpoint = endpoint::apply({
            let call_chain = call_chain.clone();
            move |_cx| {
                Ok(Ok(Session::new(MockSession {
                    call_chain: call_chain.clone(),
                })))
            }
        });
        let endpoint = session_endpoint.and_then(|session: Session<MockSession>| {
            session.with(|session| {
                session.get();
                session.set("foo");
                session.remove();
                Ok("done")
            })
        });

        endpoint
    });

    let response = runner
        .perform(Request::get("/").header("host", "localhost:3000"))
        .unwrap();
    assert!(!response.headers().contains_key("set-cookie"));

    assert_eq!(
        call_chain.result(),
        vec![Op::Get, Op::Set("foo".into()), Op::Remove, Op::Write,]
    );
}
