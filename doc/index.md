---
---

Finchers is a combinator library written in Rust, for building asynchronous HTTP services in type-safe and declarative way.

# Features

* Building an HTTP application in a declarative way
* Type safe routing
* Asynchronous handling based on Futures and Tokio

# Example

```rust
#[macro_use]
extern crate finchers;

use finchers::prelude::*;
use finchers::endpoint::prelude::*;
use finchers::service::{Server, backend};

fn main() {
    let endpoint = endpoint("api/v1").with(choice![
        get(path()).map(|id: u64| format!("GET: id={}", id)),
        post(body()).map(|body: String| format!("POST: body={}", body)),
    ]);

    let service = endpoint.into_service::<String>();
    Server::from_service(service).run(backend::default());
}
```

# Documentation

* [Users Guide][users-guide]
* [API Doc (released)][doc-released]
* [API Doc (master)][doc-master]

[doc-released]: https://docs.rs/finchers
[doc-master]: ./api/finchers/index.html
[users-guide]: ./guide/index.html
