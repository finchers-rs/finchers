# `finchers`

[![crates.io][crates-io-badge]][crates-io]
[![dependency status][dependencies-badge]][dependencies]
[![Gitter][gitter-badge]][gitter]

`finchers` is a combinator library for building asynchronous HTTP services.

The concept and design was highly inspired by [`finch`](https://github.com/finagle/finch).

## Features

* Building an HTTP service by *combining* the primitive components
* Type-safe routing
* Asynchronous handling based on Futures and Hyper 0.11
* Focuses on stable channel

## Documentation

* [User Guide][user-guide]
* [API documentation (released)][released-api]
* [API documentation (master)][master-api]

## Example

```rust,no_run
#[macro_use]
extern crate finchers;

use finchers::prelude::*;
use finchers::endpoint::prelude::*;
use finchers::output::Display;
use finchers::runtime::Server;

fn main() {
    let endpoint = path("api/v1").right(choice![
        get(param()).map(|id: u64| format!("GET: id={}", id)),
        post(body()).map(|data: String| format!("POST: body={}", data)),
    ])
    .map(Display::new);

    let service = endpoint.into_service();
    Server::new(service).run();
}
```

## Status

| Travis CI | Appveyor | Coveralls |
|:---------:|:--------:|:---------:|
| [![Travis CI][travis-badge]][travis] | [![Appveyor][appveyor-badge]][appveyor] | [![Coveralls][coveralls-badge]][coveralls] |


## License
Dual licensed under the MIT and Apache 2.0.

<!-- links -->

[user-guide]: https://finchers-rs.github.io/guide
[crates-io]: https://crates.io/crates/finchers
[released-api]: https://docs.rs/finchers/*/finchers
[master-api]: https://finchers-rs.github.io/api/finchers/
[gitter]: https://gitter.im/finchers-rs/finchers?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge
[travis]: https://travis-ci.org/finchers-rs/finchers
[appveyor]: https://ci.appveyor.com/project/ubnt-intrepid/finchers/branch/master
[coveralls]: https://coveralls.io/github/finchers-rs/finchers
[dependencies]: https://deps.rs/repo/github/finchers-rs/finchers

[crates-io-badge]: https://img.shields.io/crates/v/finchers.svg
[docs-rs-badge]: https://docs.rs/finchers/badge.svg
[master-api-badge]: https://img.shields.io/badge/docs-master-red.svg
[gitter-badge]: https://badges.gitter.im/finchers-rs/finchers.svg
[travis-badge]: https://travis-ci.org/finchers-rs/finchers.svg?branch=master
[appveyor-badge]: https://ci.appveyor.com/api/projects/status/76smoc919fni4n6l/branch/master?svg=true
[coveralls-badge]: https://coveralls.io/repos/github/finchers-rs/finchers/badge.svg
[dependencies-badge]: https://deps.rs/repo/github/finchers-rs/finchers/status.svg
