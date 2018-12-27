#[derive(Default)]
pub struct Context {
    _priv: (),
}

impl juniper::Context for Context {}

pub struct Query {
    _priv: (),
}

juniper::graphql_object!(Query: Context |&self| {
    field apiVersion() -> &str {
        "1.0"
    }
});

pub type Schema = juniper::RootNode<'static, Query, juniper::EmptyMutation<Context>>;

pub fn create_schema() -> Schema {
    Schema::new(Query { _priv: () }, juniper::EmptyMutation::new())
}
