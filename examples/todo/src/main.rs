#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate finchers;
#[macro_use]
extern crate serde;

mod application;
mod db;

use self::Response::*;
use self::db::*;
use finchers::error::NoRoute;
use finchers::json::{json_body, JsonResponder};
use finchers::prelude::*;
use finchers::runtime::Server;

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

        let find_todo = get(path())
            .and_then({
                let app = app.clone();
                move |id| app.find_todo(id)
            })
            .and_then(|todo| todo.map(TheTodo).ok_or_else(|| NoRoute::new()));

        let list_todos = get(()).and_then({
            let app = app.clone();
            move |_| app.list_todos().map(Todos)
        });

        let add_todo = post(json_body()).and_then({
            let app = app.clone();
            move |new_todo| app.add_todo(new_todo).map(NewTodo)
        });

        let patch_todo = patch((path(), json_body()))
            .and_then({
                let app = app.clone();
                move |(id, patch)| app.patch_todo(id, patch)
            })
            .and_then(|todo| todo.map(TheTodo).ok_or_else(|| NoRoute::new()));

        let delete_todo = delete(path())
            .and_then({
                let app = app.clone();
                move |id| app.delete_todo(id)
            })
            .and_then(|todo| todo.map(|_| Deleted).ok_or_else(|| NoRoute::new()));

        endpoint("api/v1/todos").with(choice![find_todo, list_todos, add_todo, patch_todo, delete_todo,])
    };

    let service = endpoint.with_responder(JsonResponder::<Response>::default());
    Server::from_service(service).run();
}
