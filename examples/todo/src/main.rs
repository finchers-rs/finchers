#![feature(conservative_impl_trait)]

#[macro_use]
extern crate finchers;
extern crate finchers_json;
#[macro_use]
extern crate serde;

mod model;
mod service;

use finchers::{Endpoint, EndpointServiceExt};
use finchers::application::Application;
use finchers_json::{json_body, JsonResponder};
use self::model::*;
use self::service::*;
use self::Response::*;

#[derive(Debug, Serialize, HttpStatus)]
#[serde(untagged)]
enum Response {
    TheTodo(Todo),
    Todos(Vec<Todo>),
    #[status_code = "CREATED"]
    Created(Todo),
    #[status_code = "NO_CONTENT"]
    Deleted,
}

fn build_endpoint(service: Service) -> impl Endpoint<Item = Option<Response>> + 'static {
    use finchers::endpoint::prelude::*;

    let find_todo = get(path())
        .and_then(service.find_todo())
        .map(|todo| todo.map(TheTodo));

    let list_todos = get(())
        .and_then(service.list_todos())
        .map(|todos| Some(Todos(todos)));

    let add_todo = post(json_body())
        .and_then(service.add_todo())
        .map(|todo| Some(Created(todo)));

    let patch_todo = patch((path(), json_body()))
        .and_then(service.apply_patch())
        .map(|todo| todo.map(TheTodo));

    let delete_todo = delete(path())
        .and_then(service.delete_todo())
        .map(|_| Some(Deleted));

    endpoint("api/v1/todos").with(choice![
        find_todo,
        list_todos,
        add_todo,
        patch_todo,
        delete_todo,
    ])
}

fn main() {
    let service = new_service();
    let endpoint = build_endpoint(service);

    Application::from_service(endpoint.with_responder(JsonResponder::<Response>::default())).run();
}
