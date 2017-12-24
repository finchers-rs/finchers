#![allow(missing_docs)]

pub(crate) mod and_then;
pub(crate) mod body;
pub(crate) mod chain;
pub(crate) mod context;
pub(crate) mod inspect;
pub(crate) mod join;
pub(crate) mod join_all;
pub(crate) mod map_err;
pub(crate) mod map;
pub(crate) mod or;
pub(crate) mod or_else;
pub(crate) mod task;
pub(crate) mod then;

pub use self::context::TaskContext;
pub use self::task::Task;
