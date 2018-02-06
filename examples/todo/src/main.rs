#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate finchers;
extern crate finchers_json;
#[macro_use]
extern crate serde;

mod db;
mod application;

use finchers::prelude::*;
use finchers::service::{backend, Server};
use finchers_json::{json_body, JsonResponder};
use self::db::*;
use self::Response::*;

#[derive(Debug, Serialize, HttpStatus)]
#[serde(untagged)]
enum Response {
    TheTodo(Todo),
    Todos(Vec<Todo>),
    #[status_code = "CREATED"]
    NewTodo(Todo),
    #[status_code = "NO_CONTENT"]
    Deleted,
}

fn main() {
    let app = application::new();

    let endpoint = {
        use finchers::endpoint::prelude::*;

        let find_todo = get(path()).and_then({
            let app = app.clone();
            move |id| app.find_todo(id).map(|todo| todo.map(TheTodo))
        });

        let list_todos = get(()).and_then({
            let app = app.clone();
            move |_| app.list_todos().map(|todos| Some(Todos(todos)))
        });

        let add_todo = post(json_body()).and_then({
            let app = app.clone();
            move |new_todo| app.add_todo(new_todo).map(|todo| Some(NewTodo(todo)))
        });

        let patch_todo = patch((path(), json_body())).and_then({
            let app = app.clone();
            move |(id, patch)| app.patch_todo(id, patch).map(|todo| todo.map(TheTodo))
        });

        let delete_todo = delete(path()).and_then({
            let app = app.clone();
            move |id| app.delete_todo(id).map(|s| s.map(|_| Deleted))
        });

        endpoint("api/v1/todos").with(choice![
            find_todo,
            list_todos,
            add_todo,
            patch_todo,
            delete_todo,
        ])
    };

    let service = endpoint.with_responder(JsonResponder::<Response>::default());
    Server::from_service(service).run(backend::default());
}
