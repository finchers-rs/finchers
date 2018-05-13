# Quickstart

> [draft] This chapter is currently working in progress.

Welcome to the users guide for Finchers.

## Installing Rust

You will need to install the latest version of Rust toolchain before starting to write a Finchers application.
The recommended way to manage Rust toolchains is using the official toolchain manager `rustup`, which could be installed as follows:

```shell-session
$ curl -sSf https://sh.rustup.rs | sh
```

Finchers requires the stable version of Rust, 1.23 or higher.

## Running Examples

The most easy way to start experimenting Finchers applications is to clone the repository of the project and to run contained examples.

For example, you will run the simple example as follows:

```shell-session
$ git clone https://github.com/finchers-rs/finchers.git
$ cd finchers
$ cargo run -p todo
```

More examples are located in the directory [`examples/`][examples].

[examples]: https://github.com/finchers-rs/finchers/tree/master/examples/