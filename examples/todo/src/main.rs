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
use finchers::json::{json_body, JsonOutput};
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
        use finchers::endpoint::ok;
        use finchers::endpoint::prelude::*;

        let find_todo = get(path())
            .try_abort({
                let app = app.clone();
                move |id| app.find_todo(id)
            })
            .try_abort(|todo: Option<_>| todo.map(TheTodo).ok_or_else(|| NoRoute::new()));

        let list_todos = get(ok(())).try_abort({
            let app = app.clone();
            move |_| app.list_todos().map(Todos)
        });

        let add_todo = post(json_body()).try_abort({
            let app = app.clone();
            move |new_todo| app.add_todo(new_todo).map(NewTodo)
        });

        let patch_todo = patch(path().and(json_body()))
            .try_abort({
                let app = app.clone();
                move |(id, patch)| app.patch_todo(id, patch)
            })
            .try_abort(|todo: Option<_>| todo.map(TheTodo).ok_or_else(|| NoRoute::new()));

        let delete_todo = delete(path())
            .try_abort({
                let app = app.clone();
                move |id| app.delete_todo(id)
            })
            .try_abort(|todo: Option<_>| todo.map(|_| Deleted).ok_or_else(|| NoRoute::new()));

        endpoint("api/v1/todos")
            .right(choice![find_todo, list_todos, add_todo, patch_todo, delete_todo,])
            .map(JsonOutput::new)
    };

    let service = endpoint.into_service();
    Server::new(service).run();
}
