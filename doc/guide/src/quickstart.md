# Quickstart

> [draft] The document is currently working in progress.

Welcome to the User Guide for Finchers.

## Installing Rust

Before starting to write an application, you will need to install the Rust toolchain to your local machine.
Finchers requires the stable version of Rust, 1.23 or higher.

## Running Examples
The examples of Finchers locate in the directory [`examples/`][examples].
For example, you could run a simple ToDo service using Finchers as follows:

```shell-session
$ git clone https://github.com/finchers-rs/finchers.git
$ cd finchers
$ cargo run -p example-todo
```

[examples]: https://github.com/finchers-rs/finchers/tree/master/examples/