# `finchers`

[![Crates.io][crates-io-badge]][crates-io]
[![Crates.io (Downloads)][downloads-badge]][crates-io]
[![Docs.rs][docs-rs-badge]][docs-rs]
[![dependency status][dependencies-badge]][dependencies]
[![Gitter][gitter-badge]][gitter]

`finchers` is a combinator library for building asynchronous HTTP services.

The concept and design was highly inspired by [`finch`](https://github.com/finagle/finch).

# Features

* Asynchronous handling powerd by futures and Tokio
* Building an HTTP service by *combining* the primitive components
* Type-safe routing without (unstable) procedural macros

# Usage

Add this item to `Cargo.toml` in your project:

```toml
[dependencies]
finchers = "0.12"
```

# Resources

* [API documentation][api-docs]
* [Examples][examples]
* [Gitter chat][gitter]

# Contributed Features

* [CORS support](https://github.com/finchers-rs/finchers-cors)
* [GraphQL integration (uses `juniper`)](https://github.com/finchers-rs/finchers-juniper)
* [WebSocket (uses `tungstenite`)](https://github.com/finchers-rs/finchers-tungstenite)
* [Session support](https://github.com/finchers-rs/finchers-session)
* [Template engine support](https://github.com/finchers-rs/finchers-template)

# Status

| Travis CI | Appveyor | Coveralls |
|:---------:|:--------:|:---------:|
| [![Travis CI][travis-badge]][travis] | [![Appveyor][appveyor-badge]][appveyor] | [![Coveralls][coveralls-badge]][coveralls] |


# License
This project is licensed under either of

* MIT license, ([LICENSE-MIT](./LICENSE-MIT) or http://opensource.org/licenses/MIT)
* Apache License, Version 2.0 ([LICENSE-APACHE](./LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)

at your option.

<!-- links -->

[crates-io]: https://crates.io/crates/finchers
[docs-rs]: https://docs.rs/finchers
[api-docs]: https://finchers-rs.github.io/docs
[examples]: https://github.com/finchers-rs/examples
[user-guide]: https://finchers-rs.github.io/finchers/guide/index.html
[gitter]: https://gitter.im/finchers-rs/finchers
[travis]: https://travis-ci.org/finchers-rs/finchers
[appveyor]: https://ci.appveyor.com/project/ubnt-intrepid/finchers/branch/master
[coveralls]: https://coveralls.io/github/finchers-rs/finchers
[dependencies]: https://deps.rs/crate/finchers/0.12.2

[crates-io-badge]: https://img.shields.io/crates/v/finchers.svg
[downloads-badge]: https://img.shields.io/crates/d/finchers.svg
[docs-rs-badge]: https://docs.rs/finchers/badge.svg
[gitter-badge]: https://badges.gitter.im/finchers-rs/finchers.svg
[travis-badge]: https://travis-ci.org/finchers-rs/finchers.svg?branch=master
[appveyor-badge]: https://ci.appveyor.com/api/projects/status/76smoc919fni4n6l/branch/master?svg=true
[coveralls-badge]: https://coveralls.io/repos/github/finchers-rs/finchers/badge.svg
[dependencies-badge]: https://deps.rs/crate/finchers/0.12.2/status.svg
