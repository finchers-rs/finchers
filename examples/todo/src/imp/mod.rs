mod model;
mod service;

use finchers::{Application, Endpoint};
use finchers::core::HttpResponse;
use finchers::handler::OptionalHandler;
use finchers::service::FinchersService;
use finchers_json::{json_body, JsonResponder};
use http::StatusCode;
use self::model::*;
use self::service::*;
use self::Response::*;

#[derive(Debug, Serialize)]
#[serde(untagged)]
enum Response {
    TheTodo(Todo),
    Todos(Vec<Todo>),
    Created(Todo),
    Deleted,
}

// TODO: code generation
impl HttpResponse for Response {
    fn status_code(&self) -> StatusCode {
        match *self {
            TheTodo(..) | Todos(..) => StatusCode::OK,
            Created(..) => StatusCode::CREATED,
            Deleted => StatusCode::NO_CONTENT,
        }
    }
}

fn build_endpoint(service: Service) -> impl Endpoint<Item = Response> + 'static {
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

pub fn main() {
    let service = new_service();
    let endpoint = build_endpoint(service);

    Application::from_service(FinchersService::new(
        endpoint,
        OptionalHandler::default(),
        JsonResponder::default(),
    )).run();
}
