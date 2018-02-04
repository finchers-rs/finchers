Finchers is a combinator library of Rust, for building *asynchronous* HTTP services.

# Features

* building an HTTP application in declarative way
* type safe routing
* asynchronous handling based on Hyper and Futures

# Hello, World

```rust
extern crate finchers;

use finchers::Endpoint;
use finchers::endpoint::path;
use finchers::endpoint::method::get;
use finchers::service::ServerBuilder;

fn main() {
    // GET /hello/:name
    let endpoint = get(("hello", path()))
        .and_then(|(_, name): (_, String)| -> Result<_, ApiError> {
            Ok(format!("Hello, {}", name))
        });

    ServerBuilder::default()
        .serve(Arc::new(endpoint));
}


enum ApiError { ... }
impl From<finchers::NoRoute> for ApiError { ... }
impl From<std::string::ParseError> for ApiError { ... }
impl finchers::ErrorResponder for ApiError { ... }
```

# Resources

* [Repository][repository]
* [API Doc (released)][doc-released]
* [API Doc (master)][doc-master]
* [User Guide][user-guide]

[repository]: https://github.com/finchers-rs/finchers
[doc-released]: https://docs.rs/finchers
[doc-master]: ./api/finchers/index.html
[user-guide]: ./guide/index.html
