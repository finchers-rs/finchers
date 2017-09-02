# `finchers`
[![Build Status](https://travis-ci.org/finchers-rs/finchers.svg?branch=master)](https://travis-ci.org/finchers-rs/finchers)
[![crates.io](https://img.shields.io/crates/v/finchers.svg)](https://crates.io/crates/finchers)
[![docs-rs](https://docs.rs/finchers/badge.svg)](https://docs.rs/finchers)

`finchers` is a combinator library for building HTTP services, based on [`hyper`](https://github.com/hyperium/hyper) and [`futures`](https://github.com/alexcrichton/futures-rs).

The concept and design of this library is highly inspired by [`finch`](https://github.com/finagle/finch) and [`combine`](https://github.com/Marwes/combine).

## Features
* ease of use
* asynchronous handling based on Futures and Hyper 0.11
* type-safe routing
* focuse on stable channel

## Example

```rust
extern crate finchers;

use finchers::{Endpoint, Json};
use finchers::endpoint::PathConst;
use finchers::endpoint::method::get;
use finchers::server::Server;

fn main() {
    // create an endpoint
    let endpoint = |_: &_| {
        get("hello".with(String::PATH)).map(|name| {
            Json(format!("Hello, {}", name))
        })
    };

    // start a HTTP server.
    Server::new(endpoint).bind("127.0.0.1:3000").run_http();
}
```

More examples are located in [`examples/`](examples/).

## Documentation
* [released](https://docs.rs/finchers/)
* [master](https://finchers-rs.github.io/finchers/finchers/index.html)

## License
Dual licensed under the MIT and Apache 2.0.
