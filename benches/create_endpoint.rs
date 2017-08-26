#![feature(test)]

extern crate finchers;
extern crate test;
extern crate tokio_core;

#[cfg(test)]
mod benches {
    use finchers::{Endpoint, NewEndpoint};
    use finchers::endpoint::method::get;
    use test::Bencher;
    use tokio_core::reactor::Core;

    #[bench]
    fn simple(b: &mut Bencher) {
        let endpoint = |_: &_| {
            let s = "foo".to_owned();
            get(s.with("bar")).map(|_| Ok("Hello") as Result<_, ()>)
        };
        run_bench(b, endpoint);
    }

    fn run_bench<E: NewEndpoint>(b: &mut Bencher, endpoint: E) {
        let core = Core::new().unwrap();
        let handle = core.handle();

        b.iter(|| { let _ = endpoint.new_endpoint(&handle); });
    }
}
