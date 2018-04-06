# `finchers`
[![Travis Build Status](https://travis-ci.org/finchers-rs/finchers.svg?branch=master)](https://travis-ci.org/finchers-rs/finchers)
[![Appveyor Build status](https://ci.appveyor.com/api/projects/status/76smoc919fni4n6l/branch/master?svg=true)](https://ci.appveyor.com/project/ubnt-intrepid/finchers/branch/master)
[![Coverage Status](https://coveralls.io/repos/github/finchers-rs/finchers/badge.svg)](https://coveralls.io/github/finchers-rs/finchers)
[![crates.io](https://img.shields.io/crates/v/finchers.svg)](https://crates.io/crates/finchers)
[![Released API docs](https://docs.rs/finchers/badge.svg)](https://docs.rs/finchers)
[![Master API docs](https://img.shields.io/badge/docs-master-red.svg)](https://finchers-rs.github.io/api/finchers/)
[![Gitter](https://badges.gitter.im/finchers-rs/finchers.svg)](https://gitter.im/finchers-rs/finchers?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge)

`finchers` is a combinator library for building asynchronous HTTP services.

The concept and design was highly inspired by [`finch`](https://github.com/finagle/finch).

## Features
* building an HTTP service by *combining* the primitive components
* type-safe routing
* asynchronous handling based on Futures and Hyper 0.11
* focuses on stable channel

## Example

```rust,no_run
#[macro_use]
extern crate finchers;

use finchers::prelude::*;
use finchers::endpoint::prelude::*;
use finchers::output::Display;
use finchers::runtime::Server;

fn main() {
    let endpoint = endpoint("api/v1").with(choice![
        get(path::<u64>()).map(|id| format!("GET: id={}", id)),
        post(body::<String>()).map(|body| format!("POST: body={}", body)),
    ]);

    let service = endpoint.map(Display::new).into_service();
    Server::new(service).run();
}
```

## Documentation
* [User Guide](https://finchers-rs.github.io/guide)
* [API doc (released)](https://docs.rs/finchers/*/finchers)
* [API doc (master)](https://finchers-rs.github.io/api/finchers/index.html)

## License
Dual licensed under the MIT and Apache 2.0.
