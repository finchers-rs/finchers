<a name="0.2.0"></a>
### 0.2.1 (2018-10-15)

* fix badge URL in README.md

<a name="0.2.0"></a>
## 0.2.0 (2018-10-09)

The initial release on this iteration.

* bump `finchers` to `0.13`
* introduce the trait `Schema` and `SharedSchema` for abstraction of `RootNode`
  - The advantages in here are as follows:
    + Simplify the trait bounds in GraphQL executors
    + `Box<RootNode>`, `Rc<RootNode>` and `Arc<RootNode>` implements `Schema`

<a name="0.1.1"></a>
### 0.1.1 (2018-10-02)
* update metadata in Cargo.toml

<a name="0.1.0"></a>
## 0.1.0 (2018-09-30)
The initial release on this iteration.
