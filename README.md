# `finchers`

[![Crates.io][crates-io-badge]][crates-io]
[![Crates.io (Downloads)][downloads-badge]][crates-io]
[![Docs.rs][docs-rs-badge]][docs-rs]
[![Master doc][master-doc-badge]][master-doc]
[![Rustc Version][rustc-version-badge]][rustc-version]
[![dependency status][dependencies-badge]][dependencies]
[![Gitter][gitter-badge]][gitter]

`finchers` is a combinator library for building asynchronous HTTP services.

The concept and design was highly inspired by [`finch`].

# Features

* Asynchronous handling powerd by futures and Tokio
* Building an HTTP service by *combining* the primitive components
* Type-safe routing without (unstable) procedural macros

# Usage

Add this item to `Cargo.toml` in your project:

```toml
[dependencies]
finchers = "0.13.4"
```

# Example

```rust,no_run
#[macro_use]
extern crate finchers;
use finchers::prelude::*;

fn main() {
    let endpoint = path!(@get / "greeting" / String)
        .map(|name: String| {
            format!("Hello, {}!\n", name)
        });

    finchers::server::start(endpoint)
        .serve("127.0.0.1:4000")
        .expect("failed to start the server");
}
```

# Resources

* [API documentation (docs.rs)][docs-rs]
* [API documentation (master)][master-doc]
* [Examples][examples]
* [Gitter chat][gitter]

# Contributed Features

* [`finchers-juniper`] - GraphQL integration support, based on [`juniper`]
* [`finchers-tungstenite`] - WebSocket support, based on [`tungstenite`]
* [`finchers-session`]: Session support
* [`finchers-template`]: Template engine support

# Status

| Travis CI | Appveyor | Codecov |
|:---------:|:--------:|:-------:|
| [![Travis CI][travis-badge]][travis] | [![Appveyor][appveyor-badge]][appveyor] | [![Codecov][codecov-badge]][codecov] |

# License
This project is licensed under either of

* MIT license, ([LICENSE-MIT](./LICENSE-MIT) or http://opensource.org/licenses/MIT)
* Apache License, Version 2.0 ([LICENSE-APACHE](./LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)

at your option.

<!-- links -->

[crates-io]: https://crates.io/crates/finchers
[docs-rs]: https://docs.rs/finchers
[master-doc]: https://finchers-rs.github.io/finchers
[examples]: https://github.com/finchers-rs/examples
[user-guide]: https://finchers-rs.github.io/finchers/guide/index.html
[gitter]: https://gitter.im/finchers-rs/finchers
[travis]: https://travis-ci.org/finchers-rs/finchers
[appveyor]: https://ci.appveyor.com/project/ubnt-intrepid/finchers/branch/master
[codecov]: https://codecov.io/gh/finchers-rs/finchers
[dependencies]: https://deps.rs/crate/finchers/0.13.4
[rustc-version]: https://rust-lang.org

[crates-io-badge]: https://img.shields.io/crates/v/finchers.svg
[downloads-badge]: https://img.shields.io/crates/d/finchers.svg
[docs-rs-badge]: https://docs.rs/finchers/badge.svg
[master-doc-badge]: https://img.shields.io/badge/docs-master-blue.svg
[gitter-badge]: https://badges.gitter.im/finchers-rs/finchers.svg
[travis-badge]: https://travis-ci.org/finchers-rs/finchers.svg?branch=master
[appveyor-badge]: https://ci.appveyor.com/api/projects/status/76smoc919fni4n6l/branch/master?svg=true
[codecov-badge]: https://codecov.io/gh/finchers-rs/finchers/branch/master/graph/badge.svg
[dependencies-badge]: https://deps.rs/crate/finchers/0.13.4/status.svg
[rustc-version-badge]: https://img.shields.io/badge/rustc-1.29+-yellow.svg

[`finchers-juniper`]: https://github.com/finchers-rs/finchers-juniper
[`finchers-tungstenite`]: https://github.com/finchers-rs/finchers-tungstenite
[`finchers-session`]: https://github.com/finchers-rs/finchers-session
[`finchers-template`]: https://github.com/finchers-rs/finchers-template

[`finch`]: https://github.com/finagle/finch
[`juniper`]: https://github.com/graphql-rust/juniper.git
[`tungstenite`]: https://github.com/snapview/tungstenite-rs
