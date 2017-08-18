# `finchers`
[![Build Status](https://travis-ci.org/ubnt-intrepid/finchers.svg?branch=master)](https://travis-ci.org/ubnt-intrepid/finchers)

`finchers` is a combinator library for building HTTP services, based on Hyper and Futures.

The concept and the design of this library is highly inspired by [`finch`](https://github.com/finagle/finch), [`combine`](https://github.com/Marwes/combine) and [`futures`](https://github.com/alexcrichton/futures-rs).

## Features
* ease of use
* asynchronous handling based on Futures and Hyper 0.11
* type-safe routing

## Example

```rust
extern crate finchers;

use finchers::Endpoint;
use finchers::combinator::method::get;
use finchers::combinator::path::{string_, end_};
use finchers::response::Json;

fn main() {
    // create a factory of endpoints.
    let new_endpoint = || {
        get(
            "hello".with(string_).skip(end_)
                .map(|name| Json(format!("Hello, {}", name)))
        )
    };

    // start a HTTP server with above factory.
    finchers::server::run_http(new_endpoint, "127.0.0.1:3000");
}
```

More examples are in [`examples/`](examples/).

## Status
Under development

## License
MIT/Apache 2.0
