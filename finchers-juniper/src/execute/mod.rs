//! GraphQL executors.
//!
//! In order to choose the strategy for executing GraphQL queries,
//! the following GraphQL executors are provided:
//!
//! * [`Nonblocking`]
//!   It spawns the task for executing GraphQL queries by using the Tokio's
//!   default executor.  It also notify the start of the blocking section to
//!   the runtime by using the blocking API provided by `tokio_threadpool`.
//!
//! * [`CurrentThread`]
//!   It executes the GraphQL queries on the current thread.
//!
//! * [`WithSpawner`]
//!   It spawn the task for executing GraphQL queries by using the specified
//!   task executor. Unlike to `Nonblocking`, it does not notify the start of
//!   blocking section to the runtime.
//!
//! [`Nonblocking`]: ./struct.Nonblocking.html
//! [`CurrentThread`]: ./struct.CurrentThread.html
//! [`WithSpawner`]: ./struct.WithSpawner.html

mod current_thread;
mod nonblocking;
mod with_spawner;

pub use self::current_thread::{current_thread, CurrentThread};
pub use self::nonblocking::{nonblocking, Nonblocking};
pub use self::with_spawner::{with_spawner, WithSpawner};

pub use self::schema::Schema;
pub use self::shared::SharedSchema;

// ====

mod schema {
    use juniper::{GraphQLType, RootNode};
    use std::rc::Rc;
    use std::sync::Arc;

    /// Trait representing a GraphQL schema.
    pub trait Schema: SchemaImpl {}

    impl<QueryT, MutationT, CtxT> Schema for RootNode<'static, QueryT, MutationT>
    where
        QueryT: GraphQLType<Context = CtxT>,
        MutationT: GraphQLType<Context = CtxT>,
    {}
    impl<S: Schema> Schema for Box<S> {}
    impl<S: Schema> Schema for Rc<S> {}
    impl<S: Schema> Schema for Arc<S> {}

    pub trait SchemaImpl {
        type Context;
        type Query: GraphQLType<Context = Self::Context>;
        type Mutation: GraphQLType<Context = Self::Context>;
        fn as_root_node(&self) -> &RootNode<'static, Self::Query, Self::Mutation>;
    }

    impl<QueryT, MutationT, CtxT> SchemaImpl for RootNode<'static, QueryT, MutationT>
    where
        QueryT: GraphQLType<Context = CtxT>,
        MutationT: GraphQLType<Context = CtxT>,
    {
        type Context = CtxT;
        type Query = QueryT;
        type Mutation = MutationT;

        #[inline]
        fn as_root_node(&self) -> &RootNode<'static, Self::Query, Self::Mutation> {
            self
        }
    }

    impl<T: Schema> SchemaImpl for Box<T> {
        type Context = T::Context;
        type Query = T::Query;
        type Mutation = T::Mutation;

        #[inline]
        fn as_root_node(&self) -> &RootNode<'static, Self::Query, Self::Mutation> {
            (**self).as_root_node()
        }
    }

    impl<T: Schema> SchemaImpl for Rc<T> {
        type Context = T::Context;
        type Query = T::Query;
        type Mutation = T::Mutation;

        #[inline]
        fn as_root_node(&self) -> &RootNode<'static, Self::Query, Self::Mutation> {
            (**self).as_root_node()
        }
    }

    impl<T: Schema> SchemaImpl for Arc<T> {
        type Context = T::Context;
        type Query = T::Query;
        type Mutation = T::Mutation;

        #[inline]
        fn as_root_node(&self) -> &RootNode<'static, Self::Query, Self::Mutation> {
            (**self).as_root_node()
        }
    }
}

mod shared {
    use super::Schema;
    use juniper::{GraphQLType, RootNode};

    /// A helper trait for representing a `Schema` which can be shared between threads.
    #[allow(missing_docs)]
    pub trait SharedSchema: SharedSchemaImpl {}

    impl<S> SharedSchema for S
    where
        S: Schema + Send + Sync + 'static,
        S::Context: Send + 'static,
        S::Query: Send + Sync + 'static,
        S::Mutation: Send + Sync + 'static,
        <S::Query as GraphQLType>::TypeInfo: Send + Sync + 'static,
        <S::Mutation as GraphQLType>::TypeInfo: Send + Sync + 'static,
    {}

    pub trait SharedSchemaImpl: Send + Sync + 'static {
        type Context: Send + 'static;
        type QueryTypeInfo: Send + Sync + 'static;
        type MutationTypeInfo: Send + Sync + 'static;
        type Query: GraphQLType<Context = Self::Context, TypeInfo = Self::QueryTypeInfo>
            + Send
            + Sync
            + 'static;
        type Mutation: GraphQLType<Context = Self::Context, TypeInfo = Self::MutationTypeInfo>
            + Send
            + Sync
            + 'static;
        fn as_root_node(&self) -> &RootNode<'static, Self::Query, Self::Mutation>;
    }

    impl<S> SharedSchemaImpl for S
    where
        S: Schema + Send + Sync + 'static,
        S::Context: Send + 'static,
        S::Query: Send + Sync + 'static,
        S::Mutation: Send + Sync + 'static,
        <S::Query as GraphQLType>::TypeInfo: Send + Sync + 'static,
        <S::Mutation as GraphQLType>::TypeInfo: Send + Sync + 'static,
    {
        type Context = S::Context;
        type Query = S::Query;
        type Mutation = S::Mutation;
        type QueryTypeInfo = <S::Query as GraphQLType>::TypeInfo;
        type MutationTypeInfo = <S::Mutation as GraphQLType>::TypeInfo;

        #[inline]
        fn as_root_node(&self) -> &RootNode<'static, Self::Query, Self::Mutation> {
            self.as_root_node()
        }
    }
}
