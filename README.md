# `finchers`
[![Build Status](https://travis-ci.org/finchers-rs/finchers.svg?branch=master)](https://travis-ci.org/finchers-rs/finchers)
[![crates.io](https://img.shields.io/crates/v/finchers.svg)](https://crates.io/crates/finchers)
[![docs-rs](https://docs.rs/finchers/badge.svg)](https://docs.rs/finchers)
[![Gitter](https://badges.gitter.im/finchers-rs/finchers.svg)](https://gitter.im/finchers-rs/finchers?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge)
<!--[![Coverage Status](https://coveralls.io/repos/github/finchers-rs/finchers/badge.svg?branch=master)](https://coveralls.io/github/finchers-rs/finchers?branch=master) -->

`finchers` is a combinator library for building asynchronous HTTP services.

## Features
* building an HTTP service by *combining* the primitive components
* type-safe routing
* asynchronous handling based on Futures and Hyper 0.11
* focuses on stable channel

The concept and design was highly inspired by [`finch`](https://github.com/finagle/finch).

## Example

```rust,no_run
#[macro_use]
extern crate finchers;

use finchers::prelude::*;
use finchers::endpoint::prelude::*;
use finchers::service::{Server, backend};

fn main() {
    let endpoint = endpoint("api/v1").with(choice![
        get(path::<u64>()).map(|id| format!("GET: id={}", id)),
        post(body::<String>()).map(|body| format!("POST: body={}", body)),
    ]);

    let service = endpoint.into_service::<String>();
    Server::from_service(service).run(backend::default());
}
```

## Documentation
* [API documentation (released)](https://docs.rs/finchers/)
* [API documentation (master)](https://finchers-rs.github.io/api/finchers/index.html)
* [Users Guide](https://finchers-rs.github.io/guide)

## License
Dual licensed under the MIT and Apache 2.0.
